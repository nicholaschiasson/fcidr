#![cfg(feature = "serde")]
#![cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]

use alloc::string::ToString;
use core::str::FromStr;

use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

use crate::{Cidr, Fcidr};

struct CidrVisitor;

impl<'de> Visitor<'de> for CidrVisitor {
    type Value = Cidr;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a cidr block")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Self::Value::from_str(v).map_err(serde::de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Cidr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(CidrVisitor)
    }
}

impl Serialize for Cidr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct FcidrVisitor;

impl<'de> Visitor<'de> for FcidrVisitor {
    type Value = Fcidr;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a list of cidr blocks")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut value = Self::Value::default();
        while let Some(element) = seq.next_element()? {
            value.union(element);
        }
        Ok(value)
    }
}

impl<'de> Deserialize<'de> for Fcidr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(FcidrVisitor)
    }
}

impl Serialize for Fcidr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.iter().count()))?;
        for element in self {
            seq.serialize_element(&element)?;
        }
        seq.end()
    }
}
