//! Whole-file view: object table, decryption key, and decoded streams.

use std::collections::HashMap;

use super::crypto::{md5, rc4};
use super::object::{Dict, Object, RawObject, scan_objects, trailer};
use super::{PdfError, Result};

const PAD: [u8; 32] = [
    0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01, 0x08,
    0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80, 0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
];

pub(super) struct PageContent<'a> {
    pub(super) streams: Vec<(u32, u16)>,
    pub(super) resources: Option<&'a Dict>,
}

pub(super) struct Document {
    pub(super) dicts: HashMap<(u32, u16), Dict>,
    streams: HashMap<(u32, u16), (usize, usize)>,
    bytes: Vec<u8>,
    file_key: Option<Vec<u8>>,
}

impl Document {
    pub(super) fn load(bytes: Vec<u8>) -> Result<Self> {
        if !bytes.starts_with(b"%PDF") {
            return Err(PdfError::NotAPdf);
        }
        let raw = scan_objects(&bytes);
        let file_key = derive_file_key(&bytes, &raw)?;

        let mut dicts = HashMap::with_capacity(raw.len());
        let mut streams = HashMap::new();
        for RawObject { id, dict, stream } in raw {
            if let Some(range) = stream {
                streams.insert(id, range);
            }
            dicts.insert(id, dict);
        }
        Ok(Self {
            dicts,
            streams,
            bytes,
            file_key,
        })
    }

    pub(super) fn dict_of<'a>(&'a self, obj: &'a Object) -> Option<&'a Dict> {
        match obj {
            Object::Dict(d) => Some(d),
            Object::Ref(n, g) => self.dicts.get(&(*n, *g)),
            _ => None,
        }
    }

    /// Decrypted and Flate-decoded payload of one stream object.
    pub(super) fn stream(&self, id: (u32, u16)) -> Option<Vec<u8>> {
        let (start, end) = *self.streams.get(&id)?;
        let raw = &self.bytes[start..end];
        let decrypted = match &self.file_key {
            Some(key) => rc4(&object_key(key, id.0, id.1), raw),
            None => raw.to_vec(),
        };
        let dict = self.dicts.get(&id)?;
        if filters(dict).contains(&"FlateDecode") {
            inflate(&decrypted)
        } else {
            Some(decrypted)
        }
    }

    /// Content streams per page, in page order, with the page's resources.
    /// Form XObjects are reached by the `Do` operator rather than listed here,
    /// so they inherit the graphics matrix in force at the invocation.
    pub(super) fn page_contents(&self) -> Vec<PageContent<'_>> {
        self.page_dicts()
            .into_iter()
            .map(|page| PageContent {
                streams: self.content_ids(page),
                resources: page.get("Resources").and_then(|o| self.dict_of(o)),
            })
            .collect()
    }

    fn content_ids(&self, page: &Dict) -> Vec<(u32, u16)> {
        match page.get("Contents") {
            Some(Object::Ref(n, g)) => vec![(*n, *g)],
            Some(Object::Array(items)) => items.iter().filter_map(Object::as_ref_id).collect(),
            _ => Vec::new(),
        }
    }

    /// Leaf `/Page` dictionaries in document order.
    fn page_dicts(&self) -> Vec<&Dict> {
        let root = self
            .dicts
            .values()
            .find(|d| d.get("Type").and_then(Object::as_name) == Some("Catalog"))
            .and_then(|d| d.get("Pages"))
            .and_then(|o| self.dict_of(o));
        let mut out = Vec::new();
        match root {
            Some(node) => self.walk_pages(node, &mut out, 0),
            None => out.extend(
                self.dicts
                    .values()
                    .filter(|d| d.get("Type").and_then(Object::as_name) == Some("Page")),
            ),
        }
        out
    }

    fn walk_pages<'a>(&'a self, node: &'a Dict, out: &mut Vec<&'a Dict>, depth: usize) {
        if depth > 32 {
            return;
        }
        match node.get("Kids").and_then(Object::as_array) {
            Some(kids) => {
                for kid in kids {
                    if let Some(child) = self.dict_of(kid) {
                        self.walk_pages(child, out, depth + 1);
                    }
                }
            }
            None => out.push(node),
        }
    }
}

fn filters(dict: &Dict) -> Vec<&str> {
    match dict.get("Filter") {
        Some(Object::Name(n)) => vec![n.as_str()],
        Some(Object::Array(items)) => items.iter().filter_map(|o| o.as_name()).collect(),
        _ => Vec::new(),
    }
}

fn inflate(data: &[u8]) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut out = Vec::new();
    if flate2::read::ZlibDecoder::new(data)
        .read_to_end(&mut out)
        .is_ok()
    {
        return Some(out);
    }
    out.clear();
    flate2::read::DeflateDecoder::new(data)
        .read_to_end(&mut out)
        .ok()
        .map(|_| out)
}

fn derive_file_key(bytes: &[u8], raw: &[RawObject]) -> Result<Option<Vec<u8>>> {
    let Some(trailer) = trailer(bytes) else {
        return Ok(None);
    };
    let Some(enc_ref) = trailer.get("Encrypt").and_then(Object::as_ref_id) else {
        return Ok(None);
    };
    let enc = raw
        .iter()
        .find(|o| o.id == enc_ref)
        .map(|o| &o.dict)
        .ok_or(PdfError::MissingEncryptDict)?;

    let v = enc.get("V").and_then(Object::as_i64).unwrap_or(0);
    let r = enc.get("R").and_then(Object::as_i64).unwrap_or(0);
    let filter = enc.get("Filter").and_then(Object::as_name).unwrap_or("");
    if filter != "Standard" || !(v == 1 || v == 2) || !(r == 2 || r == 3) {
        return Err(PdfError::UnsupportedEncryption { v, r });
    }

    let owner = enc
        .get("O")
        .and_then(Object::as_str_bytes)
        .ok_or(PdfError::MissingEncryptDict)?;
    let permissions = enc
        .get("P")
        .and_then(Object::as_i64)
        .ok_or(PdfError::MissingEncryptDict)? as i32;
    let length_bits = enc.get("Length").and_then(Object::as_i64).unwrap_or(40);
    let n = (length_bits / 8).clamp(5, 16) as usize;

    let first_id = trailer
        .get("ID")
        .and_then(Object::as_array)
        .and_then(|a| a.first())
        .and_then(Object::as_str_bytes)
        .unwrap_or(&[]);

    let mut seed = Vec::with_capacity(32 + owner.len() + 4 + first_id.len());
    seed.extend_from_slice(&PAD);
    seed.extend_from_slice(owner);
    seed.extend_from_slice(&permissions.to_le_bytes());
    seed.extend_from_slice(first_id);

    let mut key = md5(&seed).to_vec();
    if r >= 3 {
        for _ in 0..50 {
            key = md5(&key[..n]).to_vec();
        }
    }
    key.truncate(n);
    Ok(Some(key))
}

fn object_key(file_key: &[u8], num: u32, generation: u16) -> Vec<u8> {
    let mut seed = Vec::with_capacity(file_key.len() + 5);
    seed.extend_from_slice(file_key);
    seed.extend_from_slice(&num.to_le_bytes()[..3]);
    seed.extend_from_slice(&generation.to_le_bytes()[..2]);
    let digest = md5(&seed);
    digest[..(file_key.len() + 5).min(16)].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_key_follows_algorithm_1_length_rule() {
        assert_eq!(object_key(&[0u8; 5], 1, 0).len(), 10);
        assert_eq!(object_key(&[0u8; 16], 1, 0).len(), 16);
    }

    #[test]
    fn object_key_varies_by_object_number() {
        let key = [7u8; 16];
        assert_ne!(object_key(&key, 1, 0), object_key(&key, 2, 0));
        assert_ne!(object_key(&key, 1, 0), object_key(&key, 1, 1));
    }

    #[test]
    fn rejects_non_pdf_input() {
        assert!(matches!(
            Document::load(b"not a pdf".to_vec()),
            Err(PdfError::NotAPdf)
        ));
    }

    #[test]
    fn rejects_aes_encryption_rather_than_returning_nothing() {
        let src = b"%PDF-1.6\n9 0 obj\n<< /Filter /Standard /V 4 /R 4 /Length 128 /P -1 /O <00> /U <00> >>\nendobj\ntrailer\n<< /Encrypt 9 0 R /ID [<AB> <CD>] >>\n";
        assert!(matches!(
            Document::load(src.to_vec()),
            Err(PdfError::UnsupportedEncryption { v: 4, r: 4 })
        ));
    }

    #[test]
    fn unencrypted_document_loads_with_no_key() {
        let src = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog >>\nendobj\ntrailer\n<< /Size 2 >>\n";
        let doc = Document::load(src.to_vec()).unwrap();
        assert!(doc.file_key.is_none());
        assert!(doc.dicts.contains_key(&(1, 0)));
    }

    #[test]
    fn reads_an_uncompressed_stream() {
        let src = b"%PDF-1.4\n3 0 obj\n<< /Length 4 >>\nstream\nBT\nET\nendstream\nendobj\ntrailer\n<< >>\n";
        let doc = Document::load(src.to_vec()).unwrap();
        assert_eq!(doc.stream((3, 0)).unwrap(), b"BT\nET\n");
    }
}
