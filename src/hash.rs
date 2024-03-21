use anyhow::{Context, Result};
use base64::prelude::*;
use lazy_static::lazy_static;
use ring::digest::{digest, SHA256};

use crate::{get_env_typed, get_env_typed_result, pgp::AuthName};

lazy_static! {
    static ref SECRET_HASH: String =
        get_env_typed_result::<String>("SHA256_SECRET").expect("could not read");
    static ref HASH_LEN: usize = get_env_typed::<usize>("HASH_LENGTH", 12);
}

pub struct AuthHash {
    auth: String,
    verify_hash: String,
}

impl TryFrom<&str> for AuthHash {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let bytes = BASE64_STANDARD
            .decode(value)
            .context("could not base64 decode")?;
        let value = String::from_utf8(bytes).context("could not construct string")?;
        let parts: Vec<&str> = value.split(':').collect();

        let auth = *parts.first().context("invalid auth portion")?;
        let verify_hash = *parts.get(1).context("invalid hash verification")?;

        Ok(Self {
            auth: auth.to_owned(),
            verify_hash: verify_hash.to_owned(),
        })
    }
}

impl AuthHash {
    pub fn calculate_hash(&self) -> anyhow::Result<String> {
        let to_hash = format!("{}:{}", self.auth, SECRET_HASH.as_str());

        let data_hash = digest(&SHA256, to_hash.as_bytes());

        Ok(hex::encode(data_hash)[..*HASH_LEN].to_string())
    }
}

fn verify_signature(message: &AuthHash) -> Result<()> {
    let check_hash = message.calculate_hash()?;

    if message.verify_hash.eq(&check_hash) {
        Ok(())
    } else {
        anyhow::bail!("invalid hash");
    }
}
pub fn read_short_hash(message: &str) -> Result<Option<AuthName>> {
    let auth_hash: AuthHash = message.try_into().context("could not parse")?;
    verify_signature(&auth_hash).context("could not verify signature")?;

    Ok(Some(AuthName::new(&auth_hash.auth)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_reads_short_hash() {
        let result = read_short_hash("c3VwZXI6NmMyY2UwMTM3NzA3")
            .unwrap()
            .unwrap();

        assert_eq!(result.name, "super");
    }
}
