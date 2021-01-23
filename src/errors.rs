use std::convert::From;
use std::io;

#[derive(Debug)]
pub enum DocumentError {
    SyncError(io::Error),
    UserDirectories,
    NotFound,
    ParseError(io::Error),
    MetadataNotFound,
    MetadataRetrieval,
    DuplicateAttribute(String),
    AttributeType(String),
    SetError(io::Error),
}

impl From<io::Error> for DocumentError {
    fn from(err: io::Error) -> DocumentError {
        DocumentError::ParseError(err)
    }
}

impl From<()> for DocumentError {
    fn from(_: ()) -> DocumentError {
        DocumentError::MetadataNotFound
    }
}
