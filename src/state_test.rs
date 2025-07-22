use crate::authenticate::{load_token, OAuthToken};
use once_cell::sync::OnceCell;
use std::sync::Mutex;
static OAUTH_TOKEN: OnceCell<Mutex<Option<OAuthToken>>> = OnceCell::new();

pub fn set_token(token: OAuthToken) {
    OAUTH_TOKEN
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap()
        .replace(token);
}

pub fn get_token() -> Option<OAuthToken> {
    OAUTH_TOKEN.get()?.lock().unwrap().clone()
}

pub fn load_and_set_token() -> anyhow::Result<()> {
    let token = load_token()?;
    set_token(token);
    Ok(())
}
