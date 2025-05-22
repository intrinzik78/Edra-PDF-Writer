use lopdf::{Document, Object, dictionary};
use pdf_writer::{ Chunk, Content, Name, Pdf, Rect };
use serde::Deserialize;
use openssl::{
    hash::{Hasher, MessageDigest},
    pkcs7::{Pkcs7, Pkcs7Flags},
    pkcs12::Pkcs12,
    pkey::PKey,
    x509::X509,
    stack::Stack
};

use crate::{
    traits::FontType, 
    types::{ 
        AttributeField, 
        BlockType, 
        ContentField, 
        Error,
        Font, 
        FontFamily,
        FontReference,
        Page,
        PageContent,
        Style, 
        TextAlignment,
        TextBlock,
        Word,
        Writer
 }};

type Result<T> = std::result::Result<T,Error>;

/// # Main entry point of the library
#[derive(Debug,Deserialize)]
pub struct Doc {
   #[serde(rename = "type")]
   /// Deserialized from JSON `type` field: **discarded**
    pub doc_type: Option<String>,
    /// Deserialized from JSON `content` field
    pub content: Vec<ContentField>,
}

impl Doc {

    fn sign_pdf_bytes(buf: &[u8], hex_start: usize, hex_end: usize, cert: X509, pkey: PKey<openssl::pkey::Private>, chain: Stack<X509>) -> Result<Vec<u8>> {   
        let mut hasher = Hasher::new(MessageDigest::sha256())?;
        hasher.update(&buf[..hex_start])?;
        hasher.update(&buf[hex_end..])?;
        let digest = hasher.finish()?;

        let flags = Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY;
        let pkcs7 = Pkcs7::sign(
            &cert,
            &pkey,
            &chain,
            &digest,
            flags,
        )?;

        let der = pkcs7.to_der()?;
        Ok(der)
    }

    pub fn add_sig_placeholder(buf: Vec<u8>, cert: X509, pkey: PKey<openssl::pkey::Private>, chain: Stack<X509>) -> Result<Vec<u8>> {
        let mut doc = Document::load_mem(&buf)?;
        let page_id_map = doc.get_pages();   
        let page_id = match page_id_map.get(&1) {
            Some(v) => v,
            None => return Err(Error::MissingDocumentPage)
        };
        let sig_dict_id   = doc.new_object_id();
        let widget_id     = doc.new_object_id();
        let acroform_id   = doc.new_object_id();
        let placeholder_len = 8192; // bytes
        let empty_contents = vec![0u8; placeholder_len];

        let sig_dict = dictionary! {
            "Type" => Object::Name(b"Sig".to_vec()),
            "Filter" => Object::Name(b"Adobe.PPKLite".to_vec()),
            "SubFilter" => Object::Name(b"adbe.pkcs7.detached".to_vec()),
            "ByteRange" => Object::Array(vec![0.into(), 0.into(), 0.into(), 0.into()]),
            "Contents" => Object::String(empty_contents, lopdf::StringFormat::Hexadecimal),
            "Reason" => Object::String(b"User accepted terms".to_vec(), lopdf::StringFormat::Literal),
            "M"      => Object::String(b"D:20250514120000Z".to_vec(), lopdf::StringFormat::Literal),
        };
        
        doc.objects.insert(sig_dict_id, Object::Dictionary(sig_dict));   

        let widget_dict = dictionary! {
            "Type"   => Object::Name(b"Annot".to_vec()),
            "Subtype"=> Object::Name(b"Widget".to_vec()),
            "FT"     => Object::Name(b"Sig".to_vec()),
            "Rect"   => Object::Array(vec![0.into(),0.into(),0.into(),0.into()]),
            "F"      => <i32 as Into<Object>>::into(1),
            "V"      => Object::Reference(sig_dict_id),
            "P"      => Object::Reference(*page_id),
        };

        doc.objects.insert(widget_id, Object::Dictionary(widget_dict));  

        let acroform_dict = dictionary! {
            "SigFlags" => <i32 as Into<Object>>::into(3),
            "Fields"   => Object::Array(vec![Object::Reference(widget_id)]),
        };

        doc.objects.insert(acroform_id,Object::Dictionary(acroform_dict));

        let catalog_id = doc.trailer
            .get(b"Root")
            .and_then(Object::as_reference)?;

        let catalog_ref = doc.objects
            .get_mut(&catalog_id)
            .ok_or(Error::MissingDocumentObject)?;

        if let Object::Dictionary(ref mut catalog) = catalog_ref {
            catalog.set("AcroForm", Object::Reference(acroform_id));
        } else {
            return Err(Error::MissingDocumentObject);
        }
        
        let mut buf = Vec::new();
        doc.save_to(&mut buf)?;

        let contents_tag = b"/Contents<";
        let contents_pos = buf
            .windows(contents_tag.len())
            .position(|w| w == contents_tag)
            .expect("`/Contents<` not found in PDF");
        let hex_start = contents_pos + contents_tag.len();
        let hex_end   = hex_start + placeholder_len * 2; // end index of hex digits

        let br_tag = b"/ByteRange[";
        let br_pos = buf
            .windows(br_tag.len())
            .position(|w| w == br_tag)
            .expect("`/ByteRange[` not found in PDF");
        let br_start = br_pos + br_tag.len();
        let br_end = buf[br_start..]
            .iter()
            .position(|&b| b == b']')
            .expect("`]` after ByteRange not found")
            + br_start;

        let offset1 = 0;
        let offset2 = hex_start;
        let offset3 = hex_end;
        let offset4 = buf.len() - hex_end;

        let new_br = format!("{} {} {} {}", offset1, offset2, offset3, offset4);
        let mut new_br_bytes = new_br.into_bytes();
        let br_len = br_end - br_start;
        new_br_bytes.resize(br_len, b' ');

        let sig_der  = match Doc::sign_pdf_bytes(&buf, hex_start, hex_end, cert, pkey, chain) {
            Ok(der) => der,
            Err(e) => panic!("{e}")
        };

        let hex_sig: String = hex::encode(&sig_der);
        // let hex_sig_bytes = hex_sig.into_bytes();
        // hex_sig_bytes.resize(placeholder_len * 2, b'0');

        // buf[hex_start..hex_start + hex_sig_bytes.len()].copy_from_slice(&hex_sig_bytes);

        let mut doc = Document::load_mem(&buf).unwrap();
    
        if let Object::Dictionary(ref mut d) = doc.objects.get_mut(&sig_dict_id).unwrap() {
            d.set("ByteRange", Object::Array(vec![
                Object::Integer(0),
                Object::Integer(offset2 as i64),
                Object::Integer(offset3 as i64),
                Object::Integer(offset4 as i64),
            ]));
            d.set("Contents",Object::String(hex_sig.into_bytes(), lopdf::StringFormat::Hexadecimal));
        }

        let mut new_buf = Vec::new();
        doc.save_to(&mut new_buf)?;
        
        Ok(new_buf)
    }

    /// applies an offset to each line of text based on the JSON `textAlign` field
    fn apply_text_alignment(text_block: &mut TextBlock, writeable_area: f32) {

        match text_block.alignment {
            TextAlignment::Left => {},
            TextAlignment::Center => {
                for line in &mut text_block.lines {
                    line.offset = Doc::offset_center(line.width, writeable_area);
                }
            },
            TextAlignment::Right => {
                for line in &mut text_block.lines {
                    line.offset = Doc::offset_right_justify(line.width, writeable_area);
                }
            },
            TextAlignment::Justify => {
                let list_length = text_block.lines.len();

                if list_length > 2 {
                    for index in 0..(list_length -1) {
                        let line = {
                            match text_block.lines.get_mut(index) {
                                Some(line) => line,
                                None => { return }
                            }
                        };

                        if line.body.len() > 2 {
                            let offset = (writeable_area - line.width) / (line.body.len()-1) as f32;

                            for word in &mut line.body {
                                word.offset += offset;
                            }
                        }
                    }
                }
            }
        }
    }

    /// helper function for lists, returns the first text node in a content block
    fn find_first_text_node_mut (list: &mut [ContentField]) -> Option<&mut String> {
        for node in list {
            if let Some(text) = node.text.as_mut() {
                return Some(text);
            }
            if let Some(children) = node.content.as_mut() {
                if let Some(text) = Doc::find_first_text_node_mut(children) {
                    return Some(text);
                }
            }
        }

        None
    }
    
    /// returns the base font `Style` for a text block
    fn get_block_font_style(block: &ContentField) -> Style {
        let mut current_style = Style::Normal;

        if let Some(styles) = &block.style {
            let mut style_list: Vec<Style> = Vec::new();

            for style_opt in styles {
                if let Some(name) = style_opt.name() {
                    style_list.push(name);
                }
            }

            // compiled styles defined
            current_style = match style_list.contains(&Style::Bold) & style_list.contains(&Style::Italic) {
                true => Style::BoldItalic,
                false => {
                    if style_list.contains(&Style::Bold) {
                         Style::Bold
                    } else if style_list.contains(&Style::Italic) { 
                        Style::Italic
                    } else {
                        current_style
                    }
                }
            };

            if style_list.contains(&Style::Underline) {
                current_style =match current_style {
                    Style::Bold => Style::BoldUnderline,
                    Style::Italic => Style::ItalicUnderline,
                    Style::Normal => Style::Underline,
                    Style::BoldItalic => Style::BoldItalicUnderline,
                    _ => current_style
                }
            }

            if style_list.contains(&Style::Strikethrough) {
                current_style =match current_style {
                    Style::Bold => Style::BoldStrikethrough,
                    Style::Italic => Style::ItalicStrikethrough,
                    Style::Normal => Style::BoldItalicStrikethrough,
                    Style::BoldItalic => Style::Strikethrough,
                    _ => current_style
                }
            }

        }

        current_style
    }

    /// returns nested `AattributeField` for a block or section
    fn get_block_attributes(block: &ContentField) -> Option<&AttributeField> {
        if let Some(style_list) = &block.style {
            let list_length = style_list.len();

            for index in 0..list_length {
                if let Some(style) = style_list.get(index) {
                    if let Some(attribute) = &style.attributes {
                        return Some(attribute);
                    }
                } else {
                    continue
                }
            }
        }

        None
    }

    fn get_block_text_alignment(block: &ContentField) -> TextAlignment {
        block
            .attributes
            .as_ref()
            .and_then(|attribute_field| attribute_field.text_align.as_ref())
            .map(|text_alignment| match text_alignment.as_str() {
                "left" => TextAlignment::Left,
                "right" => TextAlignment::Right,
                "center" => TextAlignment::Center,
                "justify" => TextAlignment::Justify,
                _ => TextAlignment::Left
            })
            .unwrap_or(TextAlignment::Left)
    }

    /// needs a rename to 'get_header_font_size'
    fn get_block_font_size(block: &ContentField) -> f32 {
        block
            .attributes
            .as_ref()
            .and_then(|attribute_field| attribute_field.level)
            .map(|level| match level {
                1 => 16.0,
                2 => 15.0,
                3 => 14.0,
                _ => 12.0,
            })
            .unwrap_or(12.0)
    }

    /// calculates the offset required to center a line
    fn offset_center(phrase_width: f32, writeable_area: f32) -> f32 {
        if phrase_width < writeable_area {
            (writeable_area - phrase_width) / 2.0
        } else {
            0.0
        }
    }

    /// calculates the offset required to right justify a line
    fn offset_right_justify(phrase_width: f32, writeable_area: f32) -> f32 {
        if phrase_width < writeable_area {
            writeable_area - phrase_width
        } else {
            0.0
        }
    }

    /// Entry point: builds the `Writer` struct and registers pre-provided fonts, outputs a finished PDF
    pub fn render(&mut self) -> Vec<u8> {
        let mut pdf = Pdf::new();
        let mut secondary = Chunk::new();
        let mut write_head = Writer::default();


        let catalog_id = write_head.bump();
        let page_tree_id = write_head.bump();

        let times_normal = FontReference {
            label: "times-normal",
            name: Name(b"Times-Roman"),
            id: write_head.bump()
        };
        let times_bold = FontReference {
            label: "times-bold",
            name: Name(b"Times-Bold"),
            id: write_head.bump()
        };
        let times_italic = FontReference {
            label: "times-italic",
            name: Name(b"Times-Italic"),
            id: write_head.bump()
        };
        let times_bold_italic = FontReference {
            label: "times-bold-italic",
            name: Name(b"Times-BoldItalic"),
            id: write_head.bump()
        };
        
        write_head.go_to(0.0 + write_head.page_margin, write_head.page_height-write_head.page_margin);
        
        write_head.font_refs.push(times_normal);
        write_head.font_refs.push(times_bold);
        write_head.font_refs.push(times_italic);
        write_head.font_refs.push(times_bold_italic);
        write_head.font_family.insert("times-roman", Font::new());
        
        for ref_obj in write_head.font_refs.iter() {
            pdf.type1_font(ref_obj.id).base_font(ref_obj.name);
        }

        {
            for block in self.content.iter_mut() {
                match block.block_type {
                    BlockType::Heading => Doc::render_heading(&mut write_head, block),
                    BlockType::OrderedList => Doc::render_ordered_list(&mut write_head, block),
                    BlockType::Paragraph => Doc::render_paragraph(&mut write_head, block),
                    _ => {} // non block levels excluded
                }
            }

            for page in write_head.pages.iter_mut() {
                let mut pdf_page = pdf.page(page.page_id);

                pdf_page.media_box(Rect::new(0.0, 0.0, write_head.page_width, write_head.page_height));
                pdf_page.parent(page_tree_id);
                
                for content_block in page.contents.drain(..) {
                    secondary.stream(content_block.content_id, &content_block.content.finish());
                    pdf_page.contents(content_block.content_id);
                    
                    let mut obj = pdf_page.resources();
                    let mut fonts = obj.fonts();

                    for ref_obj in write_head.font_refs.iter() {
                        fonts.pair(ref_obj.name, ref_obj.id);
                    }
                }
            }
        }

        

        // let first_page_id_option = match write_head.pages.get(0) {
        //     Some(page) => Some(page.page_id),
        //     None => None
        // };

        // if let Some(first_page_id) = first_page_id_option {
        //     let sig_dict_id = write_head.bump();
        //     let sig_widget_id = write_head.bump();
        //     let acroform_id = write_head.bump();
        //     let placeholder_bytes   = 8_192;
        //     let placeholder_hex_len = placeholder_bytes * 2; // two hex chars per byte

        //     let mut sigdict = secondary.indirect(sig_dict_id).dict();
            
        //     sigdict
        //         .pair(Name(b"Filter"),    Name(b"Adobe.PPKLite"))
        //         .pair(Name(b"SubFilter"), Name(b"adbe.pkcs7.detached"));

        //     // Insert the ByteRange array and populate it with four zeroes
        //     {
        //         let mut br = sigdict.insert(Name(b"ByteRange")).array();
        //         br.item(0);
        //         br.item(0);
        //         br.item(0);
        //         br.item(0);
        //     }

        //     sigdict
        //         .pair(Name(b"Contents"), Str(&vec![b'0'; placeholder_hex_len]))
        //         .pair(Name(b"Reason"), Str(b"User accepted terms"))
        //         .pair(Name(b"M"), Str(b"D:20250514120000Z"));
        //     sigdict.finish();


        //     let mut widget = secondary.indirect(sig_widget_id).dict();
        //     widget
        //         .pair(Name(b"Type"), Name(b"Annot"))
        //         .pair(Name(b"Subtype"), Name(b"Widget"))
        //         .pair(Name(b"P"), first_page_id)
        //         .pair(Name(b"FT"), Name(b"Sig"))
        //         .pair(Name(b"T"), Str(b"Signature1"))
        //         .pair(Name(b"F"), 1); // invisible flag

        //     {
        //         let mut rect = widget.insert(Name(b"Rect")).array();
        //         rect.item(0.0);
        //         rect.item(0.0);
        //         rect.item(0.0);
        //         rect.item(0.0);
        //     }

        //     widget
        //         .pair(Name(b"V"), sig_dict_id);
        //     widget.finish();

        //     let mut form = secondary.indirect(acroform_id).dict();
        //     form
        //         .pair(Name(b"SigFlags"),
        //             (SigFlags::SIGNATURES_EXIST | SigFlags::APPEND_ONLY).bits() as i32);
        //     {
        //         let mut fields = form.insert(Name(b"Fields")).array();
        //         fields.item(sig_widget_id);
        //     }
        //     form.finish();
            
        // } else {
        //     println!("missing page_id");
        // }
         

        // append footer to each page

        // Add the ExtG states to the PDF.
        pdf.extend(&secondary);

        // Write the root of the page tree.  
        let page_iterator = write_head.pages.iter().map(|page| page.page_id);

        pdf.pages(page_tree_id)
            .kids(page_iterator)
            .count(write_head.pages.len() as i32);

        // Write the document catalog.
        pdf.catalog(catalog_id).pages(page_tree_id);


        // Finish and write the thing to a file.
        // let _ = std::fs::write("./chunks.pdf", pdf.finish());
        pdf.finish()
    }

    /// calls `render_text_block` method with no line indent
    fn render_heading(write_head: &mut Writer, block: &mut ContentField) {
        let indent: f32 = 0.0;
        let post_block_offset = 0.0;
        Doc::render_text_block(write_head, block, indent, post_block_offset);
    }

    /// calls `render_text_block` method with no line indent
    fn render_paragraph(write_head: &mut Writer, block: &mut ContentField) {
        let indent: f32 = 0.0;
        let post_block_offset = 0.0;
        Doc::render_text_block(write_head, block, indent, post_block_offset);
    }

    /// accepts a block, inserts the list number for each list item and calls `render_text_block()`
    fn render_ordered_list(write_head: &mut Writer, block: &mut ContentField) {
        let font_size = Doc::get_block_font_size(block);
        let indent = font_size;
        let mut counter = block.attributes
            .as_ref()
            .and_then(|attribute_field| attribute_field.list_start)
            .unwrap_or(1);
    
        if let Some(items) = block.content.as_mut() {
            for item in items {
                if let Some(ref mut children) = item.content {
                    if let Some(text) = Doc::find_first_text_node_mut(children) {
                        
                        let mut num_string = counter.to_string();
                        num_string.push_str(". ");

                        text.insert_str(0, &num_string);
                        
                        counter += 1;
                    }

                    for child in children {
                        let post_block_offset: f32 = font_size * 1.5;
                        Doc::render_text_block(write_head, child, indent, post_block_offset);
                    }
                }
            }
        }
    }

    /// accepts any block with a `content` field containing a `text` field`, then assembles each line of text and calls `.write()` method on the `Writer`
    /// - adds additional `Page` containers as needed
    /// - creates `Line` containers
    /// - creates `Word` containers
    /// - assembles the content into a `TextBlock` container
    /// - calls the `.write()` method with an assembled `TextBlock`
    fn render_text_block(write_head: &mut Writer, block: &ContentField, indent: f32, post_block_offset: f32) {

        if let Some(content) = &block.content {

            // basic block level styles
            let font_size = Doc::get_block_font_size(block);
            let alignment = Doc::get_block_text_alignment(block);
            let writeable_area: f32 = write_head.page_width - (write_head.page_margin * 2.0) - indent;

            // build TextBlock
            let mut text_block = TextBlock::new()
                .with_font_size(font_size)
                .and_alignment(alignment)
                .and_indent(indent);

            // build line
            let mut line = &mut text_block.lines[text_block.index];

            // iterate through each sub-section of a block assembling `TextBlock` objects and pushing them to the `Writer` for rendering to the `Content` chunk
            for section in content {

                // get the `text` field
                if let Some(text_string) = &section.text {
                    // get section level styles
                    let family = text_block.font_family;
                    let font_style  = Doc::get_block_font_style(section);
                    let attributes = Doc::get_block_attributes(section);
                    let space_width = Doc::word_width(" ", font_size, &family, &font_style, write_head);
                    
                    // iterate over each word in the section and build add `Line` object to `TextBlock` object
                    for text in text_string.split(' ') {

                        // ignore empty strings & extra spaces, perhaps should reconsider? Double spaces will not render...ignore empty strings only?
                        if text == " " || text.is_empty() { continue; }

                        let text_width: f32 = Doc::word_width(text.trim(), font_size, &family, &font_style, write_head);
                        let offset: f32 = space_width;
                        
                        // check if word will fit within the horizontal margins of a visible page
                        if (line.width + text_width + offset) > writeable_area {
                            
                            // check if line will fit within the vertical margins of a visible page & create new `Page` when necessary
                            if write_head.y - (font_size * 1.5) < write_head.page_margin {
                                Doc::build_new_page(write_head);
                            }

                            // build a new line and get a pointer to it
                            text_block.next();
                            line = &mut text_block.lines[text_block.index];
                        }

                        let word = Word {
                            attributes,
                            font_style: font_style.clone(),
                            offset,
                            text,
                            width: text_width
                        };

                        // push the built word onto the line
                        line.width += text_width + offset;
                        line.body.push(word);
                    }
                } else {
                    // executes when no text field found
                    let font_size = Doc::get_block_font_size(block);
                    let text_block = TextBlock::new()
                        .with_font_size(font_size);

                    // check if line will fit within the vertical margins of a visible page & create new `Page` when necessary
                    if write_head.y - (font_size * 1.5) < write_head.page_margin {
                        Doc::build_new_page(write_head);
                    }

                    write_head.write(text_block);
                }
            }

            Doc::apply_text_alignment(&mut text_block, writeable_area);

            write_head.write(text_block);
        } else {
            // executes when no content field found
            let font_size = Doc::get_block_font_size(block);
            let text_block = TextBlock::new()
                .with_font_size(font_size);

            // check if line will fit within the vertical margins of a visible page & create new `Page` when necessary
            if write_head.y - (font_size * 1.5) < write_head.page_margin {
                Doc::build_new_page(write_head);
            }

            write_head.write(text_block);
        }

        write_head.feed(post_block_offset);
    }

    fn build_new_page(write_head: &mut Writer) {
        let new_page_id = write_head.bump();
        let new_content_id = write_head.bump();
        let new_content = Content::new();
        let page_content = PageContent {
            content_id: new_content_id,
            content: new_content
        };

        write_head.pages.push(Page {
            page_id: new_page_id, 
            contents: Vec::from([page_content])
        });

        write_head.current_page = Some(new_page_id);
        write_head.y = write_head.page_height - write_head.page_margin;
    }

    /// helper method for `render_text_block`
    fn word_width(word: &str, font_size: f32, family: &FontFamily, font_style: &Style, write_head: &Writer) -> f32 {

        // the internals will need to be reworked to allow externally registered fonts
        let mut current_width: f32 = 0.0;

        for ch in word.chars() {
            current_width += match *family {
                    FontFamily::TimesRoman => write_head.get_char_width(&ch, font_size, font_style, "times-roman")
            };
        }

        current_width
    }
}

impl Default for Doc {
    fn default() -> Self {
        Doc {
            doc_type: None,
            content: Vec::with_capacity(20),
        }
    }
}