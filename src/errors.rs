use std::convert::From;
use std::io;

#[derive(Debug)]
pub enum Error {
    SyncError(io::Error),
    UserDirectories(String),
    ImplementationNotFound(String),
    CliError(String),
    DocumentNotFound(String),
    DocumentParseError(io::Error),
    MetadataRetrieval(String),
    MetadataNotFound(String),
    DuplicateAttribute(String),
    AttributeTypeMismatch(String),
    DirectoryReadError(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::DocumentParseError(err)
    }
}

impl From<()> for Error {
    fn from(_: ()) -> Self {
        Error::MetadataNotFound("No <meta/> tags found in document <head/>".to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
