mod date_and_time;
#[cfg(all(feature = "sqlite", feature = "serde_json"))]
mod json;
mod numeric;

use super::connection::SqliteValue;
use super::Sqlite;
use crate::deserialize::{self, FromSql, Queryable};
use crate::expression::AsExpression;
use crate::query_builder::QueryId;
use crate::serialize::{self, IsNull, Output, ToSql};
use crate::sql_types;
use crate::sql_types::SqlType;

/// The returned pointer is *only* valid for the lifetime to the argument of
/// `from_sql`. This impl is intended for uses where you want to write a new
/// impl in terms of `String`, but don't want to allocate. We have to return a
/// raw pointer instead of a reference with a lifetime due to the structure of
/// `FromSql`
#[cfg(feature = "sqlite")]
impl FromSql<sql_types::VarChar, Sqlite> for *const str {
    fn from_sql(mut value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        let text = value.read_text();
        Ok(text as *const _)
    }
}

#[cfg(feature = "sqlite")]
impl Queryable<sql_types::VarChar, Sqlite> for *const str {
    type Row = Self;

    fn build(row: Self::Row) -> deserialize::Result<Self> {
        Ok(row)
    }
}

/// The returned pointer is *only* valid for the lifetime to the argument of
/// `from_sql`. This impl is intended for uses where you want to write a new
/// impl in terms of `Vec<u8>`, but don't want to allocate. We have to return a
/// raw pointer instead of a reference with a lifetime due to the structure of
/// `FromSql`
#[cfg(feature = "sqlite")]
impl FromSql<sql_types::Binary, Sqlite> for *const [u8] {
    fn from_sql(mut bytes: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        let bytes = bytes.read_blob();
        Ok(bytes as *const _)
    }
}

#[cfg(feature = "sqlite")]
impl Queryable<sql_types::Binary, Sqlite> for *const [u8] {
    type Row = Self;

    fn build(row: Self::Row) -> deserialize::Result<Self> {
        Ok(row)
    }
}

#[cfg(feature = "sqlite")]
#[allow(clippy::cast_possible_truncation)] // we want to truncate here
impl FromSql<sql_types::SmallInt, Sqlite> for i16 {
    fn from_sql(mut value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        Ok(value.read_integer() as i16)
    }
}

#[cfg(feature = "sqlite")]
impl FromSql<sql_types::Integer, Sqlite> for i32 {
    fn from_sql(mut value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        Ok(value.read_integer())
    }
}

#[cfg(feature = "sqlite")]
impl FromSql<sql_types::Bool, Sqlite> for bool {
    fn from_sql(mut value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        Ok(value.read_integer() != 0)
    }
}

#[cfg(feature = "sqlite")]
impl FromSql<sql_types::BigInt, Sqlite> for i64 {
    fn from_sql(mut value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        Ok(value.read_long())
    }
}

#[cfg(feature = "sqlite")]
#[allow(clippy::cast_possible_truncation)] // we want to truncate here
impl FromSql<sql_types::Float, Sqlite> for f32 {
    fn from_sql(mut value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        Ok(value.read_double() as f32)
    }
}

#[cfg(feature = "sqlite")]
impl FromSql<sql_types::Double, Sqlite> for f64 {
    fn from_sql(mut value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        Ok(value.read_double())
    }
}

#[cfg(feature = "sqlite")]
impl ToSql<sql_types::Bool, Sqlite> for bool {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        let int_value = if *self { &1 } else { &0 };
        <i32 as ToSql<sql_types::Integer, Sqlite>>::to_sql(int_value, out)
    }
}

#[cfg(feature = "sqlite")]
impl ToSql<sql_types::Text, Sqlite> for str {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self);
        Ok(IsNull::No)
    }
}

#[cfg(feature = "sqlite")]
impl ToSql<sql_types::Binary, Sqlite> for [u8] {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self);
        Ok(IsNull::No)
    }
}

#[cfg(feature = "sqlite")]
impl ToSql<sql_types::SmallInt, Sqlite> for i16 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(*self as i32);
        Ok(IsNull::No)
    }
}

#[cfg(feature = "sqlite")]
impl ToSql<sql_types::Integer, Sqlite> for i32 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(*self);
        Ok(IsNull::No)
    }
}

#[cfg(feature = "sqlite")]
impl ToSql<sql_types::BigInt, Sqlite> for i64 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(*self);
        Ok(IsNull::No)
    }
}

#[cfg(feature = "sqlite")]
impl ToSql<sql_types::Float, Sqlite> for f32 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(*self as f64);
        Ok(IsNull::No)
    }
}

#[cfg(feature = "sqlite")]
impl ToSql<sql_types::Double, Sqlite> for f64 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(*self);
        Ok(IsNull::No)
    }
}

/// The SQLite timestamp with time zone type
///
/// ### [`ToSql`] impls
///
/// - [`chrono::NaiveDateTime`] with `feature = "chrono"`
/// - [`chrono::DateTime`] with `feature = "chrono"`
/// - [`time::PrimitiveDateTime`] with `feature = "time"`
/// - [`time::OffsetDateTime`] with `feature = "time"`
///
/// ### [`FromSql`] impls
///
/// - [`chrono::NaiveDateTime`] with `feature = "chrono"`
/// - [`chrono::DateTime`] with `feature = "chrono"`
/// - [`time::PrimitiveDateTime`] with `feature = "time"`
/// - [`time::OffsetDateTime`] with `feature = "time"`
///
/// [`ToSql`]: crate::serialize::ToSql
/// [`FromSql`]: crate::deserialize::FromSql
#[cfg_attr(
    feature = "chrono",
    doc = " [`chrono::NaiveDateTime`]: chrono::naive::NaiveDateTime"
)]
#[cfg_attr(
    not(feature = "chrono"),
    doc = " [`chrono::NaiveDateTime`]: https://docs.rs/chrono/0.4.19/chrono/naive/struct.NaiveDateTime.html"
)]
#[cfg_attr(feature = "chrono", doc = " [`chrono::DateTime`]: chrono::DateTime")]
#[cfg_attr(
    not(feature = "chrono"),
    doc = " [`chrono::DateTime`]: https://docs.rs/chrono/0.4.19/chrono/struct.DateTime.html"
)]
#[cfg_attr(
    feature = "time",
    doc = " [`time::PrimitiveDateTime`]: time::PrimitiveDateTime"
)]
#[cfg_attr(
    not(feature = "time"),
    doc = " [`time::PrimitiveDateTime`]: https://docs.rs/time/0.3.9/time/struct.PrimitiveDateTime.html"
)]
#[cfg_attr(
    feature = "time",
    doc = " [`time::OffsetDateTime`]: time::OffsetDateTime"
)]
#[cfg_attr(
    not(feature = "time"),
    doc = " [`time::OffsetDateTime`]: https://docs.rs/time/0.3.9/time/struct.OffsetDateTime.html"
)]
#[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
#[diesel(sqlite_type(name = "Text"))]
#[cfg(feature = "sqlite")]
pub struct Timestamptz;

/// The SQL type for JSON validation flags
///
/// This type is backed by an Integer in SQLite.
///
/// ### [`ToSql`] impls
///
/// - [`JsonValidFlag`]
///
/// [`ToSql`]: crate::serialize::ToSql
#[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
#[diesel(sqlite_type(name = "Integer"))]
#[cfg(feature = "sqlite")]
pub struct JsonValidFlags;

/// Flags for the `json_valid` function
///
/// These flags define what is meant by "well-formed" JSON when validating.
///
/// The following bits are currently defined:
/// - `0x01` → The input is text that strictly complies with canonical RFC-8259 JSON, without any extensions.
/// - `0x02` → The input is text that is JSON with JSON5 extensions.
/// - `0x04` → The input is a BLOB that superficially appears to be JSONB.
/// - `0x08` → The input is a BLOB that strictly conforms to the internal JSONB format.
///
/// By combining bits, the following useful values can be derived:
/// - `Rfc8259Json` (1) → X is RFC-8259 JSON text
/// - `Json5` (2) → X is JSON5 text
/// - `JsonbLike` (4) → X is probably JSONB
/// - `Rfc8259JsonOrJsonb` (5) → X is RFC-8259 JSON text or JSONB
/// - `Json5OrJsonb` (6) → X is JSON5 text or JSONB (recommended for most use cases)
/// - `JsonbStrict` (8) → X is strictly conforming JSONB
/// - `Rfc8259JsonOrJsonbStrict` (9) → X is RFC-8259 or strictly conforming JSONB
/// - `Json5OrJsonbStrict` (10) → X is JSON5 or strictly conforming JSONB
#[derive(Debug, Clone, Copy, AsExpression)]
#[diesel(sql_type = JsonValidFlags)]
#[cfg(feature = "sqlite")]
#[non_exhaustive]
pub enum JsonValidFlag {
    /// X is RFC-8259 JSON text (flag = 1)
    Rfc8259Json = 1,
    /// X is JSON5 text (flag = 2)
    Json5 = 2,
    /// X is probably JSONB (flag = 4)
    JsonbLike = 4,
    /// X is RFC-8259 JSON text or JSONB (flag = 5)
    Rfc8259JsonOrJsonb = 5,
    /// X is JSON5 text or JSONB (flag = 6, recommended for most use cases)
    Json5OrJsonb = 6,
    /// X is strictly conforming JSONB (flag = 8)
    JsonbStrict = 8,
    /// X is RFC-8259 or strictly conforming JSONB (flag = 9)
    Rfc8259JsonOrJsonbStrict = 9,
    /// X is JSON5 or strictly conforming JSONB (flag = 10)
    Json5OrJsonbStrict = 10,
}

#[cfg(feature = "sqlite")]
impl ToSql<JsonValidFlags, Sqlite> for JsonValidFlag {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(*self as i32);
        Ok(IsNull::No)
    }
}

// Allow i32 for backward compatibility
#[cfg(feature = "sqlite")]
impl ToSql<JsonValidFlags, Sqlite> for i32 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        <i32 as ToSql<sql_types::Integer, Sqlite>>::to_sql(self, out)
    }
}

// Allow i32 for backward compatibility
#[cfg(feature = "sqlite")]
impl AsExpression<JsonValidFlags> for i32 {
    type Expression = crate::expression::bound::Bound<JsonValidFlags, i32>;

    fn as_expression(self) -> Self::Expression {
        crate::expression::bound::Bound::new(self)
    }
}

// Allow &i32 for backward compatibility
#[cfg(feature = "sqlite")]
impl<'a> AsExpression<JsonValidFlags> for &'a i32 {
    type Expression = crate::expression::bound::Bound<JsonValidFlags, &'a i32>;

    fn as_expression(self) -> Self::Expression {
        crate::expression::bound::Bound::new(self)
    }
}
