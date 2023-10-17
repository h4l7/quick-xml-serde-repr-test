use serde_repr::{Serialize_repr, Deserialize_repr};
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ByteEnum {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct IntermediateStruct {
    #[serde(rename = "@byte")]
    pub byte: ByteEnum,
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
        }
    };
    let xml_0 = quick_xml::se::to_string_with_root("root", &root_0).unwrap();
    println!("XML_0: {xml_0:?}");
    let root_1: RootStruct = quick_xml::de::from_str(&xml_0).unwrap();
    assert_eq!(root_0, root_1);
    let xml_1 = quick_xml::se::to_string_with_root("root", &root_1).unwrap();
    println!("XML_1: {xml_1:?}");
    assert_eq!(xml_0, xml_1);
}
