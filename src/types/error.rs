use core::fmt;
use derive_more::From;

#[derive(Debug,From)]
pub enum Error {
   #[from]
    Hasher(openssl::error::ErrorStack),
   #[from]
    SaveError(std::io::Error),
   #[from]
    Lopdf(lopdf::Error),
    MissingDocumentObject,
    MissingDocumentPage
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
