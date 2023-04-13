use std::{ops::Deref, str::FromStr, fmt::Debug};
use std::io::prelude::*;

use diesel::serialize::Output;
use diesel::{deserialize::{FromSql, self}, sql_types::Uuid, pg::{Pg, PgValue}, serialize::{ToSql, self, IsNull}, FromSqlRow};
use diesel::expression::AsExpression;
use rusty_ulid::{DecodingError, Ulid};
use serde::Serialize;
use serde::Deserialize;



#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[derive(AsExpression, FromSqlRow)]
#[diesel(sql_type = Uuid)]
pub struct DieselUlid(rusty_ulid::Ulid);


impl TryFrom<&[u8]> for DieselUlid {
    type Error = DecodingError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(DieselUlid{0: rusty_ulid::Ulid::try_from(value)?})
    }
}

impl DieselUlid {
    pub fn generate() -> Self {
        DieselUlid(Ulid::generate())
    }

    pub fn as_byte_array(&self) -> [u8; 16] {
        <[u8; 16]>::from(self.0)
    }
}

impl Debug for DieselUlid {
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
        Ok(Self {0: Ulid::from_str(s)? })
    }
}

impl From<rusty_ulid::Ulid> for DieselUlid {
    fn from(value: rusty_ulid::Ulid) -> Self {
        Self {0: value}
    }
}

impl From<DieselUlid> for rusty_ulid::Ulid {
    fn from(value: DieselUlid) -> Self {
        rusty_ulid::Ulid::from(value.0)
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

#[cfg(test)]
mod tests {
    use std::{num::NonZeroU32, str::FromStr};

    use diesel::{sql_types::Uuid, pg::{Pg, PgValue, TypeOidLookup}, deserialize::FromSql};

    use crate::DieselUlid;

    #[test]
    fn conversions() {
        // String
        let ulid = DieselUlid::from_str("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
        assert_eq!(ulid.to_string(), "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string());

        // Original
        let orig_ulid = rusty_ulid::Ulid::from(ulid.clone());
        assert_eq!(orig_ulid.to_string(), "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string());

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


        let from_str = DieselUlid::from_str("7ZZZZZZZZZZZZP2RK3CHJPCC9J").unwrap();
        assert_eq!(bytes, from_str.as_byte_array())

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
        let output_uuid =
            FromSql::<Uuid, Pg>::from_sql(PgValue::new(input_uuid.as_byte_array().as_slice(), &NonZeroU32::new(5).unwrap() as &dyn TypeOidLookup)).unwrap();
        assert_eq!(input_uuid, output_uuid);
    }

    #[test]
    fn bad_uuid_from_sql() {
        let uuid = DieselUlid::from_sql(PgValue::new(b"boom", &NonZeroU32::new(5).unwrap() as &dyn TypeOidLookup));
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