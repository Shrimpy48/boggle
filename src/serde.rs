use serde::de::{Error, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::{BStr, BString, Dict};
use std::fmt;
use std::str::FromStr;

impl Serialize for BString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct BStringVisitor;

impl<'de> Visitor<'de> for BStringVisitor {
    type Value = BString;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a valid boggle string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        BString::from_str(v).map_err(|e| Error::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for BString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(BStringVisitor)
    }
}

impl Serialize for BStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for Dict {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        self.try_traverse(|s| seq.serialize_element(&s))?;
        seq.end()
    }
}

struct DictVisitor;

impl<'de> Visitor<'de> for DictVisitor {
    type Value = Dict;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a word list")
    }

    fn visit_seq<S>(self, mut access: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        let mut dict = Dict::new();
        while let Some(value) = access.next_element::<BString>()? {
            dict.insert(&value);
        }
        Ok(dict)
    }
}

impl<'de> Deserialize<'de> for Dict {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(DictVisitor)
    }
}

