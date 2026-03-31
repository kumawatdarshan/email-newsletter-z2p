use std::ops::Deref;

use anyhow::bail;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct IdempotencyKey(String);

impl TryFrom<String> for IdempotencyKey {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.is_empty() {
            bail!("Idempotency Key cannot be empty.")
        }

        let max_len = 50;

        if s.len() > max_len {
            bail!("Idempotency Key must be shorter than {max_len} characters.");
        }

        Ok(Self(s))
    }
}

impl From<IdempotencyKey> for String {
    fn from(s: IdempotencyKey) -> Self {
        s.0
    }
}

impl AsRef<str> for IdempotencyKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for IdempotencyKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
