use crate::error::{Error, Result};

struct Secrets {}
impl Secrets {
    pub fn set(key: &str, secret: String) -> Result<()> {
        keyring::Entry::new(env!("CARGO_PKG_NAME"), key)?.set_secret(secret.as_bytes())?;
        Ok(())
    }

    pub fn get(key: &str) -> Result<String> {
        let secret = keyring::Entry::new(env!("CARGO_PKG_NAME"), key)?.get_secret()?;
        String::from_utf8(secret).map_err(|e| Error::CustomError(e.to_string()))
    }
}
