//! # Introduction
//! 
//! Arde PDF Writer is a companion to the Edra Text Editor (Tsuzat) for Svelte. It accepts JSON 
//! output from the editor and converts it to a text-only PDF. Built on top of pdf_writer (Typst),
//! this is a no frills crate. It supports most of the text layout functionality found in Edra 
//! as of 2025. I make no guarantees that this will be maintained long term as it was built for
//! personal purposes. Implement pdf_writer itself (link below) rather than rely on this specific
//! crate. 
//! 
//! Feature Road Map:
//! - [X] Times Roman font family (normal,bold,italic,bold-italic)
//! - [X] Justify text blocks (left,right,center)
//! - [X] Ordered list
//! - [X] Strikethrough
//! - [X] Underline
//! - [X] Header size (H1,H2,H3)
//! - [ ] Body font size (tiny - extra large)
//! - [ ] Additional use of embedded font families
//! - [ ] Text color
//! - [ ] Text background highlight
//! - [ ] Link annotation
//! - [ ] Superscript
//! - [ ] Subscript
//! 
//! ## Links
//! PDF Writer:
//! 
//! - <https://github.com/typst/pdf-writer>
//! 
//! Edra Text Editor
//! 
//! - <https://edra.tsuzat.com/>
//! - <https://github.com/Tsuzat/Edra>
//! 
//! ## Who should use this crate?
//! Ideally nobody, but if you insist...
//!
//! **Use cases**
//! - Personal projects
//! - Simple text document rendering
//! - Used in support of Edra Text Editor
//! - JSON input from Edra
//! 
//! **Skip cases**
//! - Mission critical applications
//! - Multiple font families in document
//! - Full PDF feature set requirement (annotations, js, etc...)
//! - Embededing objects (images, audio, video, etc)
//! - HTML or Markdown input from Edra
//! 
//! # Basic Usage
//! The main entry point is the Doc struct, which is created by Serde from a string slice of JSON output
//! from Edra. The primary method available on the Doc struct is `.render()` which interfaces with pdf_writer
//! to return PDF output.
//! 
//! ### Simple render
//! ```
//! use types::Doc
//! 
//! // take a JSON string from Edra...
//! 
//! // deserialize the json string
//! let serde_content = serde_json::from_str::<Doc>(json_string_from_edra);
//! 
//! // call the .render() method on the doc struct
//! let pdf_file = match serde_content {
//!     Ok(mut doc) => doc.render(),
//!     Err(e) => panic!("{e}")
//! };
//! 
//! // ...and write the pdf to output destination
//! ```
//! pub mod types;
pub mod traits;
pub mod types;