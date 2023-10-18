use anyhow::bail;
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ByteEnum {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

impl FromStr for ByteEnum {
    // TODO real error types
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let byte: u8 = FromStr::from_str(s)?;

        match byte {
            0 => Ok(ByteEnum::Zero),
            1 => Ok(ByteEnum::One),
            2 => Ok(ByteEnum::Two),
            3 => Ok(ByteEnum::Three),
            _ => bail!("Invalid ByteEnum: {byte}."),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct IntermediateStruct {
    #[serde(deserialize_with = "flattened_xml_attr", rename = "@byte")]
    pub byte: ByteEnum,
    #[serde(deserialize_with = "flattened_xml_attr", rename = "@other")]
    pub other: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RootStruct {
    #[serde(flatten)]
    intermediate: IntermediateStruct,
}

fn main() {
    let root_0 = RootStruct {
        intermediate: IntermediateStruct {
            byte: ByteEnum::One,
            other: 1,
        },
    };
    let xml_0 = quick_xml::se::to_string_with_root("root", &root_0).unwrap();
    println!("XML_0: {xml_0:?}");
    let root_1: RootStruct = quick_xml::de::from_str(&xml_0).unwrap();
    assert_eq!(root_0, root_1);
    let xml_1 = quick_xml::se::to_string_with_root("root", &root_1).unwrap();
    println!("XML_1: {xml_1:?}");
    assert_eq!(xml_0, xml_1);
}

// https://gitlab.com/tobz1000/odata_client_rs/-/blob/f887e93d90b78529116a5b92323e920e66a60d1c/src/deserialize_with.rs
// Inspired by https://docs.rs/serde-aux/0.6.1/serde_aux/field_attributes/fn.deserialize_number_from_string.html
pub fn flattened_xml_attr<'de, D: Deserializer<'de>, T: FromXmlStr + Deserialize<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TypeOrString<T> {
        Ty(T),
        String(String),
    }

    match TypeOrString::<T>::deserialize(deserializer)? {
        TypeOrString::Ty(t) => Ok(t),
        TypeOrString::String(s) => T::from_str(&s).map_err(serde::de::Error::custom),
    }
}

/// Trait to define on types which we need to deserialize from XML within a flattened struct, for
/// which the `std::str::FromStr` is absent/unsuitable. This should mirror the behaviour of
/// serde-xml-rs for Serde data model types.
pub trait FromXmlStr: Sized {
    type Error: std::fmt::Display;
    fn from_str(s: &str) -> Result<Self, Self::Error>;
    fn deserialize_from_type<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>;
}

macro_rules! impl_from_xml_str_as_from_str {
    ($($t:ty)*) => {
        $(
            impl FromXmlStr for $t {
                type Error = <$t as std::str::FromStr>::Err;
                fn from_str(s: &str) -> Result<Self, Self::Error> {
                    s.parse()
                }

                fn deserialize_from_type<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    <$t>::deserialize(deserializer)
                }
            }
        )*
    };
}

impl_from_xml_str_as_from_str! {
    usize u8 u16 u32 u64 u128
    isize i8 i16 i32 i64 i128
    f32 f64 char ByteEnum
}

/// Can parse from "1"/"0" as well as "true"/"false".
impl FromXmlStr for bool {
    type Error = String;

    fn from_str(s: &str) -> Result<Self, Self::Error> {
        match s {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            s => Err(format!("\"{}\" is not a valid bool", s)),
        }
    }

    fn deserialize_from_type<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        bool::deserialize(deserializer)
    }
}
