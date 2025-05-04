use crate::types::{AttributeField, FontStyle};
use serde::Deserialize;

/// Deserialized from `type`field
#[derive(Debug,Deserialize,PartialEq)]
pub enum BlockType {
   #[serde(rename = "paragraph")]
    Paragraph,
   #[serde(rename = "heading")]
    Heading,
   #[serde(rename = "hardBreak")]
    Break,
   #[serde(rename = "orderedList")]
    OrderedList,
   #[serde(rename = "text")]
    Text,
   #[serde(rename = "listItem")]
    ListItem,
}

/// Deserialized from `content` field
#[allow(dead_code)]
#[derive(Debug,Deserialize)]
pub struct ContentField {
    pub content: Option<Vec<ContentField>>,     // recursive pointer to more content
   #[serde(rename = "type")]
    pub block_type: BlockType,                  // block level type (heading, p, list...)
   #[serde(rename = "marks")]
    pub style: Option<Vec<FontStyle>>,          // font style settings
   #[serde(rename = "attrs")]
    pub attributes: Option<AttributeField>,     // block level style settings
    pub text: Option<String>,                   // actual text-content node
}