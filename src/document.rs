use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use ansi_term::Colour;
use kuchiki::traits::*;
use lazycell::AtomicLazyCell;

use crate::errors::DocumentError;

const SELECTOR: &str = "head>meta";

const PREFIX: &str = "DC.";

const MULTIVALUED: &[&str] = &["Creator", "Relation.Replaces"];

#[derive(Debug, Clone)]
pub struct Document {
    id: String,
    version: i8,
    path: PathBuf,
    meta: AtomicLazyCell<Metadata>,
}

impl Document {
    pub fn from_path(path: PathBuf) -> Option<Result<Document, DocumentError>> {
        let file_name = match path.file_name() {
            Some(name) => match name.to_str() {
                Some(name) => name,
                None => return None,
            },
            None => return None,
        };
        let (id, version) = match file_name.strip_suffix(".html") {
            Some(name) => {
                let mut split = name.rsplitn(2, '-').collect::<Vec<&str>>();
                split.reverse();
                (
                    split.get(0).unwrap().to_string(),
                    -i8::from_str(split.get(1).unwrap_or(&"").to_owned()).unwrap_or(0),
                )
            }
            None => return None,
        };
        Some(Ok(Document {
            id,
            version,
            path,
            meta: AtomicLazyCell::new(),
        }))
    }

    pub fn ensure_meta(&self) -> Result<&Self, DocumentError> {
        if !self.meta.filled() {
            let html = kuchiki::parse_html().from_utf8().from_file(&self.path)?;
            let meta = Metadata::from_html(html)?;
            match self.meta.fill(meta) {
                Ok(()) => {}
                Err(val) => {
                    eprintln!(
                        "Failed to set 'meta' field for document {:?}: {:?}",
                        self, val
                    );
                    return Err(DocumentError::MetadataRetrieval);
                }
            };
        }
        Ok(self)
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn version(&self) -> &i8 {
        &self.version
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn meta(&self) -> Result<&Metadata, DocumentError> {
        Ok(&self.ensure_meta()?.meta.borrow().unwrap())
    }

    pub fn fmt_line(&self) -> Result<String, DocumentError> {
        let mut output = format!("{} ", self.path().to_str().unwrap());
        if self.id.starts_with("draft") {
            output.push_str(&format!(
                "{} (version {}) ",
                Colour::Blue.paint(self.id()),
                -self.version()
            ));
        } else {
            output.push_str(&format!(
                "{} ",
                Colour::Cyan.bold().paint(self.id().to_uppercase())
            ));
        }
        output.push_str(&format!(
            "{}",
            Colour::Black.italic().paint(self.meta()?.fmt_line())
        ));
        Ok(output)
    }

    pub fn fmt_summary(&self) -> Result<String, DocumentError> {
        let mut output = format!("{} ", self.path().to_str().unwrap());
        if self.id.starts_with("draft") {
            output.push_str(&format!(
                "{} (version {})\n\n",
                Colour::Blue.paint(self.id()),
                -self.version()
            ));
        } else {
            output.push_str(&format!(
                "{}\n\n",
                Colour::Cyan.bold().paint(self.id().to_uppercase())
            ));
        }
        output.push_str(&format!(
            "{}",
            Colour::White.italic().paint(self.meta()?.fmt_summary())
        ));
        Ok(output)
    }
}

#[derive(Debug, Clone)]
pub struct Metadata(HashMap<String, MetadataAttr>);

impl Metadata {
    fn from_html(html: kuchiki::NodeRef) -> Result<Metadata, DocumentError> {
        let mut meta = HashMap::new();
        for node in html.select(SELECTOR)? {
            let attrs = node.attributes.borrow();
            let key = match attrs.get("name") {
                Some(key) if key.starts_with(PREFIX) => {
                    key.strip_prefix(PREFIX).unwrap().to_string()
                }
                Some(_) | None => continue,
            };
            let multivalued = MULTIVALUED.contains(&(key.as_str()));
            let value = match attrs.get("content") {
                Some(value) => value.to_string(),
                None => continue,
            };
            match meta.entry(key) {
                Entry::Vacant(e) => {
                    if multivalued {
                        e.insert(MetadataAttr::Many(Vec::from([value])));
                    } else {
                        e.insert(MetadataAttr::One(value));
                    }
                }
                Entry::Occupied(mut e) => {
                    if multivalued {
                        match e.get_mut() {
                            MetadataAttr::One(_) => {
                                let msg = format!(
                                    "Expected multivalued attribute type for '{}'",
                                    e.key()
                                );
                                return Err(DocumentError::AttributeType(msg));
                            }
                            MetadataAttr::Many(values) => values.push(value),
                        }
                    } else {
                        let msg = format!("Got unexpected duplicate attribute '{}'", e.key());
                        return Err(DocumentError::DuplicateAttribute(msg));
                    }
                }
            }
        }
        Ok(Metadata(meta))
    }

    fn fmt(&self, attr_sep: &str, keyval_sep: &str, val_sep: &str, replace_nl: bool) -> String {
        (&self.0)
            .iter()
            .map(|(key, value)| -> String {
                format!(
                    "{}{}{}",
                    key,
                    keyval_sep,
                    match value {
                        MetadataAttr::One(value) => {
                            if replace_nl {
                                value.replace("\n", " ")
                            } else {
                                value.to_string()
                            }
                        }
                        MetadataAttr::Many(values) => values.join(val_sep),
                    }
                )
            })
            .collect::<Vec<String>>()
            .join(attr_sep)
    }

    fn fmt_line(&self) -> String {
        format!("<{}>", self.fmt(" // ", ": ", "; ", true))
    }

    fn fmt_summary(&self) -> String {
        self.fmt("\n\n", ":\n", ";\n", false)
    }
}

#[derive(Debug, Clone)]
pub enum MetadataAttr {
    One(String),
    Many(Vec<String>),
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test::resource_path;

    #[test]
    fn test_well_formed_rfc() -> Result<(), DocumentError> {
        let file = "rfc6468.html";
        let path = resource_path(file);
        let test_path = path.clone();
        let doc = Document::from_path(path).unwrap()?;
        assert_eq!("rfc6468", doc.id());
        assert_eq!(&0, doc.version());
        assert_eq!(&test_path, doc.path());
        let _meta = doc.meta()?;
        let strings = &[
            file,
            "RFC6468",
            "urn:ietf:rfc:6468",
            "Sieve Notification Mechanism: SIP MESSAGE",
            "Alexey Melnikov <alexey.melnikov@isode.com>",
            "February, 2012",
            "draft-melnikov-sieve-notify-sip-message",
            "This document describes a profile of the Sieve extension for",
            "notifications, to allow notifications to be sent over the SIP MESSAGE.",
        ];
        for out in &[doc.fmt_line()?, doc.fmt_summary()?] {
            for string in strings {
                assert!(out.contains(string), "'{}' not found in output", string);
            }
        }
        Ok(())
    }

    #[test]
    fn test_well_formed_draft() -> Result<(), DocumentError> {
        let file = "draft-ietf-sidrops-rpkimaxlen-05.html";
        let path = resource_path(file);
        let test_path = path.clone();
        let doc = Document::from_path(path).unwrap()?;
        assert_eq!("draft-ietf-sidrops-rpkimaxlen", doc.id());
        assert_eq!(&(-5), doc.version());
        assert_eq!(&test_path, doc.path());
        let _meta = doc.meta()?;
        let strings = &[
            file,
            "draft-ietf-sidrops-rpkimaxlen",
            "(version 5)",
            "urn:ietf:id:ietf-sidrops-rpkimaxlen",
            "The Use of Maxlength in the RPKI",
            "Snijders, Job;",
            "Gilad, Yossi;",
            "Maddison, Ben;",
            "Goldberg, Sharon;",
            "Sriram, Kotikalapudi",
            "2020-11-02",
            "draft-yossigi-rpkimaxlen",
            "This document recommends ways to reduce forged-origin hijack attack",
            "surface by prudently limiting the set of IP prefixes that are included",
            "in a Route Origin Authorization (ROA). One recommendation is to avoid",
            "using the maxLength attribute in ROAs except in some specific cases.",
            "The recommendations complement and extend those in RFC 7115. The",
            "document also discusses creation of ROAs for facilitating the use of",
            "Distributed Denial of Service (DDoS) mitigation services.",
            "Considerations related to ROAs and origin validation in the context of",
            "destination-based Remote Triggered Black Hole (RTBH) filtering are",
            "also highlighted.",
        ];
        for out in &[doc.fmt_line()?, doc.fmt_summary()?] {
            for string in strings {
                assert!(out.contains(string), "'{}' not found in output", string);
            }
        }
        Ok(())
    }

    #[test]
    fn test_bad_path() {
        let file = "...";
        let path = resource_path(file);
        let maybe_doc = Document::from_path(path);
        assert!(matches!(maybe_doc, None))
    }

    #[test]
    fn test_no_html_suffix() {
        let file = "not-found.xhtml";
        let path = resource_path(file);
        let maybe_doc = Document::from_path(path);
        assert!(matches!(maybe_doc, None))
    }

    #[test]
    fn test_not_found() {
        let file = "not-found.html";
        let path = resource_path(file);
        let maybe_doc = Document::from_path(path).unwrap().unwrap();
        assert!(matches!(
            maybe_doc.ensure_meta(),
            Err(DocumentError::ParseError(_))
        ))
    }

    #[test]
    fn test_duplicate_attributes() {
        let file = "draft-duplicates-00.html";
        let path = resource_path(file);
        let maybe_doc = Document::from_path(path).unwrap().unwrap();
        assert!(matches!(
            maybe_doc.ensure_meta(),
            Err(DocumentError::DuplicateAttribute(_))
        ))
    }
}
