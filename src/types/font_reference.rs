use pdf_writer::{ Name, Ref };

#[derive(Debug)]
pub struct FontReference<'a> {
    pub id: Ref,
    pub label: &'a str,
    pub name: Name<'a>,
}
