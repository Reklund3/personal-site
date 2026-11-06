use std::time::{Duration, SystemTime, UNIX_EPOCH};

use uuid::Uuid;

/// Max age of a form-minted UUIDv7 embedded timestamp.
pub const IDEMPOTENCY_KEY_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
/// Clock-skew allowance: keys slightly ahead of server time are still accepted.
pub const IDEMPOTENCY_KEY_FUTURE_SKEW: Duration = Duration::from_secs(60);

/// Distinguishes save vs publish so one form-minted key can cover both actions
/// under `PRIMARY KEY (idempotency_key, action)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdempotencyAction {
    SaveDraft,
    Publish,
}

impl IdempotencyAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SaveDraft => "save_draft",
            Self::Publish => "publish",
        }
    }
}

#[derive(Debug)]
pub struct IdempotencyKey(String);

impl TryFrom<String> for IdempotencyKey {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.is_empty() {
            anyhow::bail!("The idempotency key cannot be empty");
        }
        let max_length = 50;
        if s.len() >= max_length {
            anyhow::bail!(
                "The idempotency key must be shorter 
                than {max_length} characters"
            );
        }
        let uuid = Uuid::parse_str(&s)
            .map_err(|_| anyhow::anyhow!("The idempotency key must be a UUID"))?;
        if uuid.get_version_num() != 7 {
            anyhow::bail!("The idempotency key must be a UUIDv7");
        }
        let ts = uuid
            .get_timestamp()
            .ok_or_else(|| anyhow::anyhow!("The idempotency key is missing a timestamp"))?;
        let (secs, _nanos) = ts.to_unix();
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow::anyhow!("System clock before Unix epoch"))?
            .as_secs();
        if secs > now_secs.saturating_add(IDEMPOTENCY_KEY_FUTURE_SKEW.as_secs()) {
            anyhow::bail!("The idempotency key timestamp is in the future");
        }
        if now_secs.saturating_sub(secs) > IDEMPOTENCY_KEY_MAX_AGE.as_secs() {
            anyhow::bail!("The idempotency key has expired");
        }
        Ok(Self(s))
    }
}

impl From<IdempotencyKey> for String {
    fn from(k: IdempotencyKey) -> Self {
        k.0
    }
}

impl AsRef<str> for IdempotencyKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{NoContext, Timestamp};

    fn key_at(unix_secs: u64) -> String {
        Uuid::new_v7(Timestamp::from_unix(NoContext, unix_secs, 0)).to_string()
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    #[test]
    fn accepts_fresh_v7() {
        IdempotencyKey::try_from(Uuid::now_v7().to_string()).unwrap();
    }

    #[test]
    fn rejects_v4() {
        let err = IdempotencyKey::try_from(Uuid::new_v4().to_string()).unwrap_err();
        assert!(err.to_string().contains("UUIDv7"), "{err}");
    }

    #[test]
    fn rejects_nil() {
        let err = IdempotencyKey::try_from(Uuid::nil().to_string()).unwrap_err();
        assert!(err.to_string().contains("UUIDv7"), "{err}");
    }

    #[test]
    fn rejects_malformed() {
        let err = IdempotencyKey::try_from(String::from("not-a-uuid")).unwrap_err();
        assert!(err.to_string().contains("UUID"), "{err}");
    }

    #[test]
    fn rejects_stale() {
        let stale = key_at(now_secs() - IDEMPOTENCY_KEY_MAX_AGE.as_secs() - 60);
        let err = IdempotencyKey::try_from(stale).unwrap_err();
        assert!(err.to_string().contains("expired"), "{err}");
    }

    #[test]
    fn rejects_far_future() {
        let future = key_at(now_secs() + IDEMPOTENCY_KEY_FUTURE_SKEW.as_secs() + 120);
        let err = IdempotencyKey::try_from(future).unwrap_err();
        assert!(err.to_string().contains("future"), "{err}");
    }

    #[test]
    fn accepts_within_future_skew() {
        let skewed = key_at(now_secs() + IDEMPOTENCY_KEY_FUTURE_SKEW.as_secs() / 2);
        IdempotencyKey::try_from(skewed).unwrap();
    }
}
