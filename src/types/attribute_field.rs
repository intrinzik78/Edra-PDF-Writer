/// # Generated from the `attrs` JSON field
/// Attributes describe the styles of a block of text
use serde::Deserialize;

#[derive(Debug,Deserialize,PartialEq)]
#[serde(tag = "t")]
pub struct AttributeField {
   #[serde(rename = "textAlign")]
    pub text_align: Option<String>,
    pub level: Option<u8>,
    pub class: Option<String>,
    pub tight: Option<bool>,
   #[serde(rename = "start")] 
    pub list_start: Option<u8>,
    pub color: Option<String>,
   #[serde(rename = "fontSize")]
    pub font_size: Option<String>
}