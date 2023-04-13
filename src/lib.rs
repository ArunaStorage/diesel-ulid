#![forbid(unsafe_code)]

use std::fmt::Display;
use std::io::prelude::*;
use std::{fmt::Debug, ops::Deref, str::FromStr};

use diesel::expression::AsExpression;
use diesel::serialize::Output;
use diesel::{
    deserialize::{self, FromSql},
    pg::{Pg, PgValue},
    serialize::{self, IsNull, ToSql},
    sql_types::Uuid,
    FromSqlRow,
};
use rusty_ulid::{DecodingError, Ulid};
use serde::Deserialize;
use serde::Serialize;

#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash, AsExpression, FromSqlRow
)]
#[diesel(sql_type = Uuid)]
pub struct DieselUlid(rusty_ulid::Ulid);

impl DieselUlid {
    pub fn generate() -> Self {
        DieselUlid(Ulid::generate())
    }

    pub fn as_byte_array(&self) -> [u8; 16] {
        <[u8; 16]>::from(self.0)
    }
}

impl Default for DieselUlid{
    fn default() -> Self {
        DieselUlid{0: rusty_ulid::Ulid::from(0_u128)}
    }
}

impl TryFrom<&[u8]> for DieselUlid {
    type Error = DecodingError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(DieselUlid(rusty_ulid::Ulid::try_from(value)?))
    }
}

impl From<&[u8; 16]> for DieselUlid {
    fn from(value: &[u8; 16]) -> Self {
        DieselUlid(rusty_ulid::Ulid::from(*value))
    }
}

impl From<[u8; 16]> for DieselUlid {
    fn from(value: [u8; 16]) -> Self {
        DieselUlid(rusty_ulid::Ulid::from(value))
    }
}

impl Debug for DieselUlid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self)
    }
}

impl Display for DieselUlid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.to_string())
    }
}

impl Deref for DieselUlid {
    type Target = rusty_ulid::Ulid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for DieselUlid {
    type Err = DecodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

impl From<rusty_ulid::Ulid> for DieselUlid {
    fn from(value: rusty_ulid::Ulid) -> Self {
        Self(value)
    }
}

impl From<DieselUlid> for rusty_ulid::Ulid {
    fn from(value: DieselUlid) -> Self {
        value.0
    }
}

impl FromSql<Uuid, Pg> for DieselUlid {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        DieselUlid::try_from(value.as_bytes()).map_err(Into::into)
    }
}

impl ToSql<Uuid, Pg> for DieselUlid {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(&self.as_byte_array())
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

// UUID conversions
impl From<uuid::Uuid> for DieselUlid {
    fn from(value: uuid::Uuid) -> Self {
        DieselUlid::from(value.as_bytes())
    }
}

impl From<DieselUlid> for uuid::Uuid {
    fn from(value: DieselUlid) -> Self {
        uuid::Uuid::from_bytes(value.as_byte_array())
    }
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroU32, str::FromStr};

    use diesel::{
        deserialize::FromSql,
        pg::{Pg, PgValue, TypeOidLookup},
        sql_types::Uuid,
    };

    use crate::DieselUlid;

    #[test]
    fn conversions() {
        // String
        let ulid = DieselUlid::from_str("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
        assert_eq!(ulid.to_string(), "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string());

        // Original
        let orig_ulid = rusty_ulid::Ulid::from(ulid.clone());
        assert_eq!(
            orig_ulid.to_string(),
            "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()
        );

        // Back
        let into_before = DieselUlid::from(orig_ulid);
        assert_eq!(ulid, into_before);

        // Bytes
        let bytes = [
            0xFF_u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66,
            0x31, 0x32,
        ];

        let bytes_ulid = DieselUlid::try_from(bytes.as_slice()).unwrap();

        assert_eq!("7ZZZZZZZZZZZZP2RK3CHJPCC9J", &bytes_ulid.to_string());

        let direct_conv = DieselUlid::from(bytes);
        assert_eq!(bytes_ulid, direct_conv);

        let from_str = DieselUlid::from_str("7ZZZZZZZZZZZZP2RK3CHJPCC9J").unwrap();
        assert_eq!(bytes, from_str.as_byte_array())
    }

    #[test]
    fn conversions_uuid() {
        let orig_string = "67e55044-10b1-426f-9247-bb680e5fe0c8";

        let from_str = uuid::Uuid::from_str(orig_string).unwrap();

        let as_ulid = DieselUlid::from(from_str);

        let back_to_uuid = uuid::Uuid::from(as_ulid);

        assert_eq!(orig_string, back_to_uuid.to_string().as_str())
    }

    #[test]
    fn test_default() {
        let ulid = DieselUlid::default();
        let ulid_2 = DieselUlid::from(rusty_ulid::Ulid::from(0x0000_0000_0000_0000_0000_0000_0000_0000));

        assert_eq!(ulid, ulid_2)
    }

    #[test]
    fn test_generate() {
        use chrono::Utc;
        let ulid = DieselUlid::generate();
        // Should be the same millisecond
        assert!(
            ulid.datetime().timestamp_millis() - Utc::now().timestamp_millis() < 5
        )
    }

    #[test]
    fn test_debug_display() {
        let ulid = DieselUlid::generate();
        // Should be the same millisecond
        assert_eq!(
            format!("{ulid}"), format!("{:?}", ulid)
        )
    }

    #[test]
    fn test_format() {
        let ulid = DieselUlid::from_str("7ZZZZZZZZZZZZP2RK3CHJPCC9J").unwrap();

        assert_eq!(format!("{:?}", ulid), format!("7ZZZZZZZZZZZZP2RK3CHJPCC9J"))
    }

    // Can not test to_sql because diesel does not export Output::test()
    // #[test]
    // fn uuid_to_sql() {
    //     let mut buffer = Vec::new();
    //     let bytes = [
    //         0xFF_u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66,
    //         0x31, 0x32,
    //     ];
    //     let test_uuid = DieselUlid::try_from(bytes.as_slice()).unwrap();
    //     let mut bytes = Output::test(&mut buffer);
    //     ToSql::<Uuid, Pg>::to_sql(&test_uuid, &mut bytes).unwrap();
    //     assert_eq!(&buffer, test_uuid.as_byte_array().as_slice());
    // }

    #[test]
    fn some_uuid_from_sql() {
        let bytes = [
            0xFF_u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66,
            0x31, 0x32,
        ];
        let input_uuid = DieselUlid::try_from(bytes.as_slice()).unwrap();
        let output_uuid = FromSql::<Uuid, Pg>::from_sql(PgValue::new(
            input_uuid.as_byte_array().as_slice(),
            &NonZeroU32::new(5).unwrap() as &dyn TypeOidLookup,
        ))
        .unwrap();
        assert_eq!(input_uuid, output_uuid);
    }

    #[test]
    fn bad_uuid_from_sql() {
        let uuid = DieselUlid::from_sql(PgValue::new(
            b"boom",
            &NonZeroU32::new(5).unwrap() as &dyn TypeOidLookup,
        ));
        assert!(uuid.is_err());
        // The error message changes slightly between different
        // uuid versions, so we just check on the relevant parts
        // The exact error messages are either:
        // "invalid bytes length: expected 16, found 4"
        // or
        // "invalid length: expected 16 bytes, found 4"
        let error_message = uuid.unwrap_err().to_string();
        assert!(error_message.starts_with("invalid"));
        assert!(error_message.contains("length"));
    }

    #[test]
    fn no_uuid_from_sql() {
        let uuid = DieselUlid::from_nullable_sql(None);
        assert_eq!(
            uuid.unwrap_err().to_string(),
            "Unexpected null for non-null column"
        );
    }
}
