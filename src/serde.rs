#![cfg(feature = "serde")]
#![cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]

use std::str::FromStr;

use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

use crate::{Cidr, Fcidr};

struct CidrVisitor;

impl<'de> Visitor<'de> for CidrVisitor {
    type Value = Cidr;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let cidr: Cidr = serde_json::from_str("\"127.0.1.2/31\"").unwrap();
        println!("{cidr}");
        println!(
            "{}",
            serde_json::json!("128.0.0.0/30".parse::<Cidr>().unwrap())
        );
        let mut fcidr = Fcidr::new("10.0.0.0/8".parse().unwrap());
        fcidr.difference("10.128.128.127/32".parse().unwrap());
        println!("{}", serde_json::json!(fcidr));
        let fcidr: Fcidr = serde_json::from_str("[\"10.0.0.0/9\",\"10.128.0.0/17\",\"10.128.128.0/26\",\"10.128.128.64/27\",\"10.128.128.96/28\",\"10.128.128.112/29\",\"10.128.128.120/30\",\"10.128.128.124/31\",\"10.128.128.126/32\",\"10.128.128.128/25\",\"10.128.129.0/24\",\"10.128.130.0/23\",\"10.128.132.0/22\",\"10.128.136.0/21\",\"10.128.144.0/20\",\"10.128.160.0/19\",\"10.128.192.0/18\",\"10.129.0.0/16\",\"10.130.0.0/15\",\"10.132.0.0/14\",\"10.136.0.0/13\",\"10.144.0.0/12\",\"10.160.0.0/11\",\"10.192.0.0/10\"]").unwrap();
        for (i, cidr) in fcidr.iter().enumerate() {
            println!("{i} - {cidr}");
        }
    }
}
