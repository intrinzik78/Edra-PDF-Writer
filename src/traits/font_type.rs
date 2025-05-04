use crate::types::Style;

/// Registering new fonts with the document is easy, but time consuming. The character width for each character
/// must be provided for all font types supported (normal, bold, italic, bold-italic). This can be determined
/// by calculating the maximum number of characters that can fit an arbitrary page width of 499.0.
pub trait FontType {
    fn new() -> Self;
    fn normal(&self, ch: &char, font_size: f32) -> f32;
    fn bold(&self, ch: &char, font_size: f32) -> f32;
    fn italic(&self, ch: &char, font_size: f32) -> f32;
    fn bold_italic(&self, ch: &char, font_size: f32) -> f32;
    fn standardize(width: f32, font_size: f32) -> f32;

    fn char_width(&self, ch: &char, font_style: &Style, font_size: f32) -> f32 {
        match *font_style {
            // normal
            Style::Normal => self.normal(ch, font_size),
            Style::Underline => self.normal(ch, font_size),
            Style::Strikethrough => self.normal(ch, font_size),

            // bold
            Style::Bold => self.bold(ch,font_size),
            Style::BoldUnderline => self.bold(ch,font_size),
            Style::BoldStrikethrough => self.bold(ch,font_size),

            // italic
            Style::Italic => self.italic(ch,font_size),
            Style::ItalicUnderline => self.italic(ch,font_size),
            Style::ItalicStrikethrough => self.italic(ch,font_size),

            // bold & italic
            Style::BoldItalic => self.bold_italic(ch,font_size),
            Style::BoldItalicUnderline => self.bold_italic(ch,font_size),
            Style::BoldItalicStrikethrough => self.bold_italic(ch,font_size),
        }
    }
}