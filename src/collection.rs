use std::collections::{hash_map, BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;
use std::slice;
use std::vec;

use crate::document::Document;
use crate::errors::{Error, Result};

#[derive(Clone)]
pub struct Collection(Vec<Document>);

impl Collection {
    pub fn from_dir(path: PathBuf) -> Result<Self> {
        let dir = match fs::read_dir(path) {
            Ok(dir) => dir,
            Err(e) => return Err(Error::DirectoryReadError(e)),
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

    pub fn newest(&self, count: u8) -> Self {
        self.to_map().newest(count)
    }

    pub fn filter_types(&self, types: Option<Vec<&str>>) -> Self {
        match types {
            Some(types) => Collection(
                self.into_iter()
                    .filter(|&doc| types.iter().any(|t| doc.id().starts_with(t)))
                    .map(|doc| doc.to_owned())
                    .collect(),
            ),
            None => self.to_owned(),
        }
    }

    fn to_map(&self) -> CollectionMap {
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
        CollectionMap(map)
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

struct CollectionMap<'a>(HashMap<&'a String, BTreeMap<&'a i8, &'a Document>>);

impl CollectionMap<'_> {
    fn newest(self, count: u8) -> Collection {
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
    use super::*;

    use crate::test::resource_path;

    #[test]
    fn test_construct_collection() -> Result<()> {
        let path = resource_path("");
        let collection = Collection::from_dir(path)?;
        assert_eq!(collection.into_iter().count(), 4);
        Ok(())
    }

    #[test]
    fn test_newest_collection() -> Result<()> {
        let path = resource_path("");
        let newest = Collection::from_dir(path)?.newest(1);
        assert_eq!(newest.into_iter().count(), 3);
        Ok(())
    }

    #[test]
    fn test_type_filter() -> Result<()> {
        let path = resource_path("");
        let types = Some(vec!["rfc", "bcp"]);
        let filtered = Collection::from_dir(path)?.filter_types(types);
        assert_eq!(filtered.into_iter().count(), 1);
        Ok(())
    }

    #[test]
    fn test_bad_path() {
        let path = resource_path("not-found");
        let maybe_collection = Collection::from_dir(path);
        assert!(matches!(
            maybe_collection,
            Err(Error::DirectoryReadError(_))
        ))
    }
}
