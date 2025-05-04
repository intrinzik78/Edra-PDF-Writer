use pdf_writer::{ Content, Ref };

/// container for pdf_writer page references
pub struct Page {
    pub page_id: Ref,
    pub contents: Vec<PageContent>
}

/// each page gets a single `Content` object
pub struct PageContent {
    pub content_id: Ref,
    pub content: Content
}