
use crate::types::{ 
    AttributeField,
    FontFamily,
    Style,
    TextAlignment
};

/// block level container to push a `Line` ojbect into
/// ```
/// Example
/// let mut text_block = TextBlock::new()
///     .with_font_size(font_size) // f32
///     .and_alignment(alignment)  // TextAlignment
///     .and_indent(indent);       // f32
/// ```
pub struct TextBlock<'a> {
    pub alignment: TextAlignment,
    pub lines: Vec<Line<'a>>,
    pub font_family: FontFamily,
    pub font_size: f32,
    // keeps track of which `Line` is currently being pushed to by `Doc::render_text_block()`
    pub index: usize,
    pub indent: f32
}

impl TextBlock<'_> {
    /// default settings:
    /// - Font size: 12.0
    /// - Font family: Times-Roman
    /// - Text alignment: Left
    /// - Indentation: 0.0
    pub fn new() -> Self {
        TextBlock::default()
    }

    /// builder function setting font size
    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// builder function setting block alignment
    pub fn and_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// builder function setting block indentation
    pub fn and_indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    /// creates a new, empty, `Line` for `Doc::render_text_block()` to push a `Word` object into
    pub fn next(&mut self) {
        self.lines.push(Line {
            body: Vec::new(),
            width: 0.0,
            offset: 0.0
        });

        // sets the current line index so `Doc::render_text_block()` knows which `Line` to push the next `Word` to
        self.index += 1;
    }
}

/// Wrapper for `Word` objects that fit a visual page
#[derive(Debug)]
pub struct Line <'a> {
    pub body: Vec<Word<'a>>,
    pub width: f32,
    pub offset: f32,
}

/// &str container with word level styles
#[derive(Debug)]
pub struct Word <'a>{
    pub attributes: Option<&'a AttributeField>,
    pub font_style: Style,
    pub offset: f32,
    pub text: &'a str,
    pub width: f32,
}

impl Default for TextBlock<'_> {
    /// default settings:
    /// - Font size: 12.0
    /// - Font family: Times-Roman
    /// - Text alignment: Left
    /// - Indentation: 0.0
    fn default() -> Self {
        let line = Line {
            body: Vec::new(),
            width: 0.0,
            offset: 0.0
        };

        TextBlock {
            alignment: TextAlignment::Left,
            font_size: 12.0,
            font_family: FontFamily::TimesRoman,
            lines: Vec::from([line]),
            index: 0,
            indent: 0.0
        }
    }
}
