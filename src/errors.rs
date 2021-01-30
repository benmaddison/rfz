use std::convert::From;
use std::io;

#[derive(Debug)]
pub enum Error {
    AttributeTypeMismatch(String),
    CliError(String),
    DirectoryReadError(io::Error),
    DocumentNotFound(String),
    DocumentParseError(io::Error),
    DuplicateAttribute(String),
    ImplementationNotFound(String),
    MetadataNotFound(String),
    MetadataRetrieval(String),
    SyncError(io::Error),
    UserDirectories(String),
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
