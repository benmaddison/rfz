use std::collections::{hash_map, BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;
use std::slice;
use std::vec;

use crate::document::Document;
use crate::errors::DocumentError;

#[derive(Debug)]
pub struct Collection(Vec<Document>);

impl Collection {
    pub fn from_dir(path: PathBuf) -> Result<Self, DocumentError> {
        let dir = match fs::read_dir(path) {
            Ok(dir) => dir,
            Err(e) => return Err(DocumentError::SetError(e)),
        };
        let mut collection = Vec::new();
        for dir_entry in dir {
            let doc_path = match dir_entry {
                Ok(e) => e.path(),
                Err(_) => continue,
            };
            if !doc_path.is_file() {
                continue;
            }
            let doc = match Document::from_path(doc_path) {
                Some(result) => match result {
                    Ok(doc) => doc,
                    Err(e) => return Err(e),
                },
                None => continue,
            };
            collection.push(doc);
        }
        Ok(Collection(collection))
    }

    pub fn to_map(&self) -> Result<CollectionMap, DocumentError> {
        let mut map = HashMap::new();
        for doc in self {
            match map.entry(doc.id()) {
                hash_map::Entry::Vacant(e) => {
                    let mut map = BTreeMap::new();
                    map.insert(doc.version(), doc);
                    e.insert(map);
                }
                hash_map::Entry::Occupied(mut e) => {
                    let map = e.get_mut();
                    map.insert(doc.version(), doc);
                }
            };
        }
        Ok(CollectionMap(map))
    }
}

impl IntoIterator for Collection {
    type Item = Document;
    type IntoIter = vec::IntoIter<Document>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Collection {
    type Item = &'a Document;
    type IntoIter = slice::Iter<'a, Document>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).iter()
    }
}

impl<'a> IntoIterator for &'a mut Collection {
    type Item = &'a mut Document;
    type IntoIter = slice::IterMut<'a, Document>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).iter_mut()
    }
}

#[derive(Debug)]
pub struct CollectionMap<'a>(HashMap<&'a String, BTreeMap<&'a i8, &'a Document>>);

impl CollectionMap<'_> {
    pub fn newest(self, count: u8) -> Collection {
        let mut collection = Vec::new();
        for versions in self.0.values() {
            for doc in versions.values().take(count.into()) {
                collection.push(doc.to_owned().to_owned());
            }
        }
        Collection(collection)
    }
}

#[cfg(test)]
mod test {
    // use super::*;

    // use crate::test::resource_path;

    #[test]
    fn test_dummy() {
        assert_eq!(2+2, 4)
    }
}
