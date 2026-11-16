//! Binds an idempotency key to the request it was first used for.
//!
//! A [`RequestFingerprint`] is stored (as a response header, inside the existing
//! saved-response headers) with every saved outcome. On replay, the fingerprint
//! of the new request is compared with the stored one; a mismatch is refused.
use actix_web::http::header::{HeaderName, HeaderValue};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Response header carrying the stored fingerprint. Stripped before any saved
/// response is handed back to a caller.
pub(crate) const FINGERPRINT_HEADER: HeaderName =
    HeaderName::from_static("x-idempotency-request-fingerprint");

const VERSION_PREFIX: &str = "v1:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestFingerprint(String);

/// Result of comparing a new request against a stored fingerprint value.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FingerprintCheck {
    Match,
    Mismatch,
    /// Stored value missing or not in the `v1:<64 lowercase hex>` form.
    Legacy,
}

impl RequestFingerprint {
    /// SHA-256 over a domain tag, then each field as an 8-byte big-endian length
    /// followed by its bytes. Fields are hashed exactly as given.
    fn from_fields(fields: &[&str]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"idempotency-request-v1\0");
        for field in fields {
            hasher.update((field.len() as u64).to_be_bytes());
            hasher.update(field.as_bytes());
        }
        let hex: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Self(format!("{VERSION_PREFIX}{hex}"))
    }

    pub fn save_draft(
        issue_id: Uuid,
        loaded_content_hash: &str,
        title: &str,
        text_content: &str,
        html_content: &str,
    ) -> Self {
        Self::from_fields(&[
            "save_draft",
            &issue_id.hyphenated().to_string(),
            loaded_content_hash,
            title,
            text_content,
            html_content,
        ])
    }

    /// `issue_id` is `None` when the form carried no id (absent or empty).
    pub fn compose_publish(
        issue_id: Option<Uuid>,
        title: &str,
        text_content: &str,
        html_content: &str,
    ) -> Self {
        let id = issue_id.map(|id| id.hyphenated().to_string());
        Self::from_fields(&[
            "compose_publish",
            id.as_deref().unwrap_or(""),
            title,
            text_content,
            html_content,
        ])
    }

    pub fn publish_saved(issue_id: Uuid) -> Self {
        Self::from_fields(&["publish_saved", &issue_id.hyphenated().to_string()])
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn header_value(&self) -> HeaderValue {
        HeaderValue::from_str(&self.0).expect("fingerprint is ASCII")
    }

    pub(crate) fn check(&self, stored: Option<&HeaderValue>) -> FingerprintCheck {
        let Some(stored) = stored.and_then(|v| v.to_str().ok()) else {
            return FingerprintCheck::Legacy;
        };
        let valid = stored.strip_prefix(VERSION_PREFIX).is_some_and(|hex| {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        });
        if !valid {
            FingerprintCheck::Legacy
        } else if stored == self.0 {
            FingerprintCheck::Match
        } else {
            FingerprintCheck::Mismatch
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(title: &str, text: &str, html: &str) -> RequestFingerprint {
        RequestFingerprint::save_draft(Uuid::nil(), "", title, text, html)
    }

    #[test]
    fn format_is_versioned_lowercase_hex() {
        let f = draft("t", "x", "h");
        let hex = f.as_str().strip_prefix("v1:").unwrap();
        assert_eq!(hex.len(), 64);
        assert!(
            hex.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        );
        assert_eq!(f, draft("t", "x", "h"));
    }

    #[test]
    fn field_boundaries_are_unambiguous() {
        assert_ne!(draft("ab", "c", ""), draft("a", "bc", ""));
        assert_ne!(draft("", "", "a"), draft("a", "", ""));
        assert_ne!(
            RequestFingerprint::compose_publish(None, "ab", "c", ""),
            RequestFingerprint::compose_publish(None, "a", "bc", "")
        );
    }

    #[test]
    fn endpoints_are_separated() {
        let id = Uuid::new_v4();
        let all = [
            RequestFingerprint::save_draft(id, "", "t", "x", "h"),
            RequestFingerprint::compose_publish(Some(id), "t", "x", "h"),
            RequestFingerprint::publish_saved(id),
        ];
        assert_ne!(all[0], all[1]);
        assert_ne!(all[0], all[2]);
        assert_ne!(all[1], all[2]);
    }

    #[test]
    fn different_ids_and_content_differ() {
        assert_ne!(
            RequestFingerprint::publish_saved(Uuid::new_v4()),
            RequestFingerprint::publish_saved(Uuid::new_v4())
        );
        assert_ne!(
            RequestFingerprint::save_draft(Uuid::nil(), "", "t", "x", "h"),
            RequestFingerprint::save_draft(Uuid::nil(), "", "t", "x", "h2")
        );
        assert_ne!(
            RequestFingerprint::save_draft(Uuid::nil(), "", "t", "x", "h"),
            RequestFingerprint::save_draft(Uuid::nil(), &"a".repeat(64), "t", "x", "h")
        );
    }

    #[test]
    fn absent_compose_id_none_equals_empty_but_not_a_real_id() {
        // The handler maps both None and "" to None, so they share one value.
        let none = RequestFingerprint::compose_publish(None, "t", "x", "h");
        assert_eq!(
            none,
            RequestFingerprint::compose_publish(None, "t", "x", "h")
        );
        assert_ne!(
            none,
            RequestFingerprint::compose_publish(Some(Uuid::nil()), "t", "x", "h")
        );
    }

    #[test]
    fn ids_are_canonicalised() {
        let lower = "67e55044-10b1-426f-9247-bb680e5fe0c8";
        let a = Uuid::parse_str(lower).unwrap();
        let b = Uuid::parse_str(&lower.to_uppercase()).unwrap();
        assert_eq!(
            RequestFingerprint::publish_saved(a),
            RequestFingerprint::publish_saved(b)
        );
        assert_eq!(
            RequestFingerprint::save_draft(a, "", "t", "x", "h"),
            RequestFingerprint::save_draft(b, "", "t", "x", "h")
        );
    }

    #[test]
    fn check_classifies_stored_values() {
        let f = draft("t", "x", "h");
        let other = draft("t", "x", "other");
        assert_eq!(f.check(Some(&f.header_value())), FingerprintCheck::Match);
        assert_eq!(
            f.check(Some(&other.header_value())),
            FingerprintCheck::Mismatch
        );
        assert_eq!(f.check(None), FingerprintCheck::Legacy);
        let bad = [
            "",
            "garbage",
            "v2:0000000000000000000000000000000000000000000000000000000000000000",
            "v1:abc",
        ];
        for s in bad {
            assert_eq!(
                f.check(Some(&HeaderValue::from_str(s).unwrap())),
                FingerprintCheck::Legacy,
                "{s}"
            );
        }
    }
}
