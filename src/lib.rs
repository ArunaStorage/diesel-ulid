#![forbid(unsafe_code)]

use bytes::BufMut;
#[cfg(feature = "diesel")]
use diesel::expression::AsExpression;
#[cfg(feature = "diesel")]
use diesel::serialize::Output;
#[cfg(feature = "diesel")]
use diesel::{
    deserialize::{self, FromSql},
    pg::{Pg, PgValue},
    serialize::{self, IsNull, ToSql},
    sql_types::Uuid as DieselUUID,
    FromSqlRow,
};
#[cfg(feature = "postgres")]
use postgres_types::private::BytesMut;
#[cfg(feature = "postgres")]
use postgres_types::{accepts, FromSql as PgFromSql, ToSql as PgToSql, Type};
use rusty_ulid::{DecodingError, Ulid};
use serde::de::{self, Unexpected, Visitor};
use serde::Serialize;
use serde::{Deserialize, Deserializer};
use std::error::Error;
use std::fmt::{self, Display};
#[cfg(feature = "diesel")]
use std::io::Write;
use std::{fmt::Debug, ops::Deref, str::FromStr};
use uuid::Uuid;

#[cfg(feature = "diesel")]
#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Hash, AsExpression, FromSqlRow,
)]
#[diesel(sql_type = DieselUUID)]
pub struct DieselUlid(rusty_ulid::Ulid);

#[cfg(feature = "postgres")]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Hash)]
pub struct DieselUlid(rusty_ulid::Ulid);

impl DieselUlid {
    pub fn generate() -> Self {
        DieselUlid(Ulid::generate())
    }

    pub fn as_byte_array(&self) -> [u8; 16] {
        <[u8; 16]>::from(self.0)
    }
}

#[cfg(feature = "postgres")]
impl<'a> PgFromSql<'a> for DieselUlid {
    fn from_sql(_: &Type, raw: &[u8]) -> Result<DieselUlid, Box<dyn Error + Sync + Send>> {
        Ok(DieselUlid::try_from(raw)?)
    }
    accepts!(UUID);
}

impl<'de> Deserialize<'de> for DieselUlid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DieselUlidVisitor;

        impl<'de> Visitor<'de> for DieselUlidVisitor {
            type Value = DieselUlid;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a string or bytes containing a diesel_ulid as uuid or ulid"
                )
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match DieselUlid::from_str(s) {
                    Ok(v) => Ok(v),
                    Err(_) => match Uuid::from_str(s) {
                        Ok(v) => Ok(DieselUlid::from(v)),
                        Err(e) => Err(de::Error::invalid_value(
                            Unexpected::Str(s),
                            &e.to_string().as_str(),
                        )),
                    },
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&v)
            }

            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_bytes(&v.to_be_bytes())
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v.len() == 16 {
                    return match DieselUlid::try_from(v) {
                        Ok(v) => Ok(v),
                        Err(e) => Err(de::Error::invalid_value(
                            Unexpected::Bytes(v),
                            &e.to_string().as_str(),
                        )),
                    };
                } else {
                    self.visit_str(std::str::from_utf8(v).map_err(|e| {
                        de::Error::invalid_value(Unexpected::Bytes(v), &e.to_string().as_str())
                    })?)
                }
            }
        }
        deserializer.deserialize_bytes(DieselUlidVisitor)
    }
}

#[cfg(feature = "postgres")]
impl PgToSql for DieselUlid {
    fn to_sql(
        &self,
        _: &Type,
        w: &mut BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn Error + Sync + Send>> {
        w.put_slice(&self.as_byte_array());
        Ok(postgres_types::IsNull::No)
    }
    accepts!(UUID);
    postgres_types::to_sql_checked!();
}

impl Default for DieselUlid {
    fn default() -> Self {
        DieselUlid(rusty_ulid::Ulid::from(0_u128))
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

#[cfg(feature = "diesel")]
impl FromSql<DieselUUID, Pg> for DieselUlid {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        DieselUlid::try_from(value.as_bytes()).map_err(Into::into)
    }
}

#[cfg(feature = "diesel")]
impl ToSql<DieselUUID, Pg> for DieselUlid {
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
    use std::{collections::HashMap, str::FromStr};

    #[cfg(feature = "diesel")]
    use diesel::{
        deserialize::FromSql,
        pg::{Pg, PgValue, TypeOidLookup},
        sql_types::Uuid as DieselUUID,
    };
    #[cfg(feature = "diesel")]
    use std::num::NonZeroU32;

    use crate::DieselUlid;

    #[test]
    fn conversions() {
        // String
        let ulid = DieselUlid::from_str("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
        assert_eq!(ulid.to_string(), "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string());

        // Original
        let orig_ulid = rusty_ulid::Ulid::from(ulid);
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
    fn conversions_uuid_test_serde() {
        let orig_string = r#"{"test" : "67e55044-10b1-426f-9247-bb680e5fe0c8"}"#;

        let from_json: HashMap<String, DieselUlid> = serde_json::from_str(orig_string).unwrap();

        assert_eq!(
            from_json.get("test").unwrap().to_string().as_str(),
            "37WN84845H89QS4HXVD075ZR68"
        )
    }

    #[test]
    fn conversions_uuid_test_serde_bin() {
        let ulid = DieselUlid::generate();

        let serialized = bincode::serialize(&ulid).unwrap();

        let deserialized: DieselUlid = bincode::deserialize(&serialized).unwrap();

        assert_eq!(ulid, deserialized)
    }

    #[test]
    fn test_default() {
        let ulid = DieselUlid::default();
        let ulid_2 = DieselUlid::from(rusty_ulid::Ulid::from(
            0x0000_0000_0000_0000_0000_0000_0000_0000,
        ));

        assert_eq!(ulid, ulid_2)
    }

    #[test]
    fn test_generate() {
        use chrono::Utc;
        let ulid = DieselUlid::generate();
        // Should be the same millisecond
        assert!(ulid.datetime().timestamp_millis() - Utc::now().timestamp_millis() < 5)
    }

    #[test]
    fn test_debug_display() {
        let ulid = DieselUlid::generate();
        // Should be the same millisecond
        assert_eq!(format!("{ulid}"), format!("{:?}", ulid))
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
    #[cfg(feature = "diesel")]
    fn some_uuid_from_sql() {
        let bytes = [
            0xFF_u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66,
            0x31, 0x32,
        ];
        let input_uuid = DieselUlid::try_from(bytes.as_slice()).unwrap();
        let output_uuid = FromSql::<DieselUUID, Pg>::from_sql(PgValue::new(
            input_uuid.as_byte_array().as_slice(),
            &NonZeroU32::new(5).unwrap() as &dyn TypeOidLookup,
        ))
        .unwrap();
        assert_eq!(input_uuid, output_uuid);
    }

    #[test]
    #[cfg(feature = "diesel")]
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
    #[cfg(feature = "diesel")]
    fn no_uuid_from_sql() {
        let uuid = DieselUlid::from_nullable_sql(None);
        assert_eq!(
            uuid.unwrap_err().to_string(),
            "Unexpected null for non-null column"
        );
    }
}
