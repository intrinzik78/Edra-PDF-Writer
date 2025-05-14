use std::collections::HashMap;

use pdf_writer::{ Content, Str, Ref };

use crate::{
    traits::FontType, 
    types::{ 
        Font,
        FontReference,
        Page,
        PageContent, 
        Style,
        TextBlock
}};

/// the rendering engine
/// contains
/// - page references
/// - ref allocator
/// - font family mapping
/// - default page settings
pub struct Writer<'a> {
    pub x: f32,
    pub y: f32,
    pub alloc: Ref,
    pub current_page: Option<Ref>,
    pub font_refs: Vec<FontReference<'a>>,
    pub font_family: HashMap<&'a str,Font>,
    pub pages: Vec<Page>,
    pub page_height: f32,
    pub page_width: f32,
    pub page_margin: f32,
}

/// Sets the write head to x: 0, y: 0, embeds the Times-Roman font, instantiates the Ref Allocator
impl Default for Writer<'_> {
    fn default() -> Self {
        let mut alloc = Ref::new(1);
        let mut contents:Vec<PageContent> = Vec::new();
        let mut pages: Vec<Page> = Vec::with_capacity(1);

        let page_height: f32 = 842.4;
        let page_width: f32 = 595.6;
        let page_margin: f32 = 48.0;
        let first_page_ref = alloc.bump();
        let first_content_ref = alloc.bump();
        let content_obj = Content::new();

        let page_content = PageContent {
            content_id: first_content_ref,
            content: content_obj
        };

        contents.push(page_content);

        let page = Page {
            page_id: first_page_ref,
            contents,
        };

        pages.push(page);

        Writer {
            x: 0.0,
            y: 0.0,
            alloc,
            current_page: Some(first_page_ref),
            font_refs: Vec::with_capacity(4),
            font_family: HashMap::with_capacity(1),
            pages,
            page_height,
            page_width,
            page_margin,
        }
    }   
}

impl Writer <'_> {
    /// get a new reference for indirect object
    pub fn bump(&mut self) -> Ref {
        self.alloc.bump()
    }

    /// scrolls the writer down the page
    pub fn feed(&mut self, num: f32) {
        self.y -= num;
    }

    /// moves the writer to a new position
    pub fn go_to(&mut self, num_x: f32, num_y: f32) {
        self.x = num_x;
        self.y = num_y;
    }

    /// does the heavy lifting of rendering the `TextBlock` to `self.current_page`
    pub fn write(&mut self, text_block: TextBlock) {
        // a page must exist by now
        debug_assert!(self.pages.is_empty() == false);
        // fonts must exist by now
        debug_assert!(self.font_refs.is_empty() == false);

        let block_indent = text_block.indent;
        let mut font_map: HashMap<&str, &FontReference> = HashMap::with_capacity(self.font_refs.len());
        
        for font in self.font_refs.iter() {
            font_map.insert(font.label,font);
        }

        for line in text_block.lines.iter() {

            // line break
            if line.body.is_empty() {
                self.y -= 1.5 * text_block.font_size;
                continue;
            }

            self.x = 0.0;
            self.x += block_indent;
            self.x += self.page_margin;
            self.x += line.offset;

            debug_assert!(self.x >= self.page_margin);
            debug_assert!(self.x <= self.page_width - self.page_margin);
            debug_assert!(self.y >= self.page_margin);
            debug_assert!(self.y <= self.page_height - self.page_margin);

            let page_opt = &mut self.pages.last_mut();

            if let Some(page) = page_opt {
                if let Some(content) = page.contents.pop() {
                    let mut target = content.content;

                    target.begin_text();
                    target.next_line(self.x, self.y);

                    let line_start_index = self.x;
            
                    for word in &line.body {
                        // a `Word`` object can't have empty text
                        // if it is, there is likely a bug in `Doc::render_text_block()`
                        debug_assert!(word.text.is_empty() == false);
                        debug_assert!(self.x >= self.page_margin);
                        debug_assert!(self.x <= self.page_width - self.page_margin);

                        match word.font_style {
                            Style::Normal => if let Some(ref_obj) = font_map.get("times-normal") { target.set_font(ref_obj.name, text_block.font_size); },
                            Style::Underline => if let Some(ref_obj) = font_map.get("times-normal") { target.set_font(ref_obj.name, text_block.font_size); },
                            Style::Strikethrough => if let Some(ref_obj) =  font_map.get("times-normal") { target.set_font(ref_obj.name, text_block.font_size); },
                            Style::Italic => if let Some(ref_obj) =  font_map.get("times-italic") { target.set_font(ref_obj.name, text_block.font_size); },
                            Style::ItalicUnderline => if let Some(ref_obj) =  font_map.get("times-italic") { target.set_font(ref_obj.name, text_block.font_size); },
                            Style::ItalicStrikethrough => if let Some(ref_obj) =  font_map.get("times-italic") { target.set_font(ref_obj.name, text_block.font_size); },
                            Style::Bold => if let Some(ref_obj) =  font_map.get("times-bold") { target.set_font(ref_obj.name, text_block.font_size); },
                            Style::BoldUnderline => if let Some(ref_obj) =  font_map.get("times-bold") { target.set_font(ref_obj.name, text_block.font_size); },
                            Style::BoldStrikethrough => if let Some(ref_obj) =  font_map.get("times-bold") { target.set_font(ref_obj.name, text_block.font_size); },
                            Style::BoldItalicUnderline => if let Some(ref_obj) =  font_map.get("times-bold-italic") { target.set_font(ref_obj.name, text_block.font_size); },
                            Style::BoldItalicStrikethrough => if let Some(ref_obj) =  font_map.get("times-bold-italic") { target.set_font(ref_obj.name, text_block.font_size); },
                            Style::BoldItalic => if let Some(ref_obj) =  font_map.get("times-bold-italic") { target.set_font(ref_obj.name, text_block.font_size); }
                        };

                        target.show(Str(word.text.as_bytes()));
                        target.next_line(word.width + word.offset, 0.0);

                        self.x += word.width + word.offset;
                    }

                    let ending_index = self.x;

                    /* *************************** */

                    self.x = line_start_index;
                    target.move_to(self.x, self.y);
                    
                    let mut underline_flag = false;
                    let mut strikethrough_flag = false;
                    let mut underline_points:Vec<f32> = Vec::with_capacity(10);
                    let mut strikethrough_points:Vec<f32> = Vec::with_capacity(2);
                    let mut last_offset = 0.0;

                    for word in &line.body {
                        debug_assert!(self.x >= self.page_margin);
                        debug_assert!(self.x <= self.page_width - self.page_margin);

                        match word.font_style {
                            Style::Underline => {
                                if !underline_flag {
                                    underline_points.push(self.x);
                                    underline_flag = true;
                                }
                            },
                            Style::ItalicUnderline => {
                                if !underline_flag {
                                    underline_points.push(self.x);
                                    underline_flag = true;
                                }
                            },
                            Style::BoldUnderline => {
                                if !underline_flag {
                                    underline_points.push(self.x);
                                    underline_flag = true;
                                }
                            },
                            Style::BoldItalicUnderline => {
                                if !underline_flag {
                                    underline_points.push(self.x);
                                    underline_flag = true;
                                }
                            },
                            _=> {
                                if underline_flag {
                                    underline_points.push(self.x - word.offset);
                                    underline_flag = false;
                                }
                            }
                        }

                        match word.font_style {
                            Style::Strikethrough => {
                                if !strikethrough_flag {
                                    strikethrough_points.push(self.x);
                                    strikethrough_flag = true;
                                }
                            },
                            Style::BoldStrikethrough => {
                                if !strikethrough_flag {
                                    strikethrough_points.push(self.x);
                                    strikethrough_flag = true;
                                }
                            },
                            Style::ItalicStrikethrough => {
                                if !strikethrough_flag {
                                    strikethrough_points.push(self.x);
                                    strikethrough_flag = true;
                                }
                            },
                            Style::BoldItalicStrikethrough => {
                                if !strikethrough_flag {
                                    strikethrough_points.push(self.x);
                                    strikethrough_flag = true;
                                }
                            },
                            _=> {
                                if strikethrough_flag {
                                    strikethrough_points.push(self.x - word.offset);
                                    strikethrough_flag = false;
                                }
                            }
                        }

                        last_offset = word.offset;

                        self.x += word.offset;
                        self.x += word.width;
                    }

                    if underline_flag {
                        underline_points.push(self.x - last_offset);
                    }

                    if strikethrough_flag {
                        strikethrough_points.push(self.x - last_offset);
                    }

                    if underline_points.len() > 1 {
                        for (index,point) in underline_points.iter().enumerate() {
                            let even = index % 2 == 0;

                            if even {
                                target.move_to(*point, self.y - (text_block.font_size / 3.3));
                            } else {
                                target.line_to(*point, self.y - (text_block.font_size / 3.3));
                            }
                        }
                    }

                    if strikethrough_points.len() > 1 {
                        for (index,point) in strikethrough_points.iter().enumerate() {
                            let even = index % 2 == 0;

                            if even {
                                target.move_to(*point, self.y - (text_block.font_size / 3.3));
                            } else {
                                target.line_to(*point, self.y - (text_block.font_size / 3.3));
                            }
                        }
                    }

                    target.move_to(ending_index, self.y);

                    self.y -= text_block.font_size * 1.5;

                    target.stroke();
                    target.end_text();

                    let new_content = PageContent {
                        content_id: content.content_id,
                        content: target
                    };

                    page.contents.push(new_content);
                }
            }
        }
    }

    pub fn get_char_width(&self, ch: &char,  font_size: f32, font_style: &Style, search_string: &str) -> f32 {
        const DEFAULT_FONT_WIDTH: f32 = 55.0;
        const DEFAULT_FONT_SIZE: f32 = 18.0;

        // vefify search string returns an existing font family
        debug_assert!(self.font_family.contains_key(search_string));

        if let Some(font) = self.font_family.get(search_string) {
            font.char_width(ch, font_style, font_size)
        } else {
            font_size / DEFAULT_FONT_SIZE * DEFAULT_FONT_WIDTH
        }
    }
}
