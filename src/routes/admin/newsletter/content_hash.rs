//! Stale-tab guard for Save draft: the edit page embeds a hash of the content it
//! loaded; a save is accepted only if the stored row still hashes to that value.
use sha2::{Digest, Sha256};

/// Hidden form field carrying the loaded-content hash.
pub(crate) const LOADED_HASH_FIELD: &str = "loaded_content_hash";

/// SHA-256 over a domain tag plus each field as an 8-byte big-endian byte
/// length followed by its UTF-8 bytes (title, text, html), hex-encoded.
/// Length prefixes make the encoding unambiguous (`"ab","c"` ≠ `"a","bc"`).
pub(crate) fn draft_content_hash(title: &str, text_content: &str, html_content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"newsletter-draft-content-v1\0");
    for field in [title, text_content, html_content] {
        hasher.update((field.len() as u64).to_be_bytes());
        hasher.update(field.as_bytes());
    }
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// What the submitting form expected to find.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ExpectedContent {
    /// Empty field (compose page): expect no stored row yet.
    NoRow,
    /// Edit page: expect a stored row whose content hashes to this value.
    Hash(String),
}

/// Parse the submitted field; anything but empty or 64 lowercase hex is an error.
pub(crate) fn parse_expected(raw: &str) -> Result<ExpectedContent, &'static str> {
    if raw.is_empty() {
        return Ok(ExpectedContent::NoRow);
    }
    if raw.len() == 64
        && raw
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(ExpectedContent::Hash(raw.to_owned()))
    } else {
        Err("malformed loaded_content_hash")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_hex_and_length_prefixed() {
        let h = draft_content_hash("t", "x", "<p>x</p>");
        assert_eq!(h.len(), 64);
        assert_eq!(h, draft_content_hash("t", "x", "<p>x</p>"));
        assert_ne!(
            draft_content_hash("ab", "c", ""),
            draft_content_hash("a", "bc", "")
        );
        assert_ne!(
            draft_content_hash("", "", "a"),
            draft_content_hash("a", "", "")
        );
    }

    #[test]
    fn parse_expected_accepts_empty_and_hex_only() {
        assert_eq!(parse_expected(""), Ok(ExpectedContent::NoRow));
        let h = draft_content_hash("a", "b", "c");
        assert_eq!(parse_expected(&h), Ok(ExpectedContent::Hash(h.clone())));
        assert!(parse_expected("zz").is_err());
        assert!(parse_expected(&h.to_uppercase()).is_err());
        assert!(parse_expected(&format!("{h}0")).is_err());
    }
}
