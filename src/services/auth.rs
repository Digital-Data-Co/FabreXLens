use crate::services::api::AuthContext;
use dialoguer::{theme::ColorfulTheme, Input, Password};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CredentialDomain {
    FabreX,
    Gryf,
    Supernode,
    Redfish,
}

impl fmt::Display for CredentialDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialDomain::FabreX => write!(f, "FabreX"),
            CredentialDomain::Gryf => write!(f, "Gryf"),
            CredentialDomain::Supernode => write!(f, "Supernode"),
            CredentialDomain::Redfish => write!(f, "Redfish"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CredentialKey {
    domain: CredentialDomain,
    scope: String,
}

impl CredentialKey {
    pub fn new(domain: CredentialDomain, scope: impl Into<String>) -> Self {
        Self {
            domain,
            scope: scope.into(),
        }
    }

    pub fn default(domain: CredentialDomain) -> Self {
        Self::new(domain, "default")
    }

    pub fn domain(&self) -> &CredentialDomain {
        &self.domain
    }

    pub fn scope(&self) -> &str {
        &self.scope
    }

    fn storage_key(&self) -> String {
        format!("{}::{}", self.domain, self.scope)
    }
}

impl fmt::Display for CredentialKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.domain, self.scope)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSecret {
    pub username: String,
    pub password: String,
    pub api_token: Option<String>,
}

impl CredentialSecret {
    pub fn redacted_summary(&self) -> String {
        format!(
            "{} / {}",
            self.username,
            self.api_token
                .as_deref()
                .map(|_| "•••• API token")
                .unwrap_or("no API token")
        )
    }

    pub fn as_auth_context(&self) -> AuthContext {
        if let Some(token) = &self.api_token {
            AuthContext::bearer(token.clone())
        } else {
            AuthContext::basic(self.username.clone(), self.password.clone())
        }
    }
}

pub trait CredentialStore: Send + Sync {
    fn save(&self, key: &CredentialKey, secret: &CredentialSecret) -> Result<(), AuthError>;
    fn get(&self, key: &CredentialKey) -> Result<Option<CredentialSecret>, AuthError>;
    fn delete(&self, key: &CredentialKey) -> Result<(), AuthError>;
}

pub struct KeyringCredentialStore {
    service_name: String,
}

impl KeyringCredentialStore {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    fn entry(&self, key: &CredentialKey) -> Result<Entry, AuthError> {
        Entry::new(&self.service_name, &key.storage_key()).map_err(AuthError::Keyring)
    }
}

impl CredentialStore for KeyringCredentialStore {
    fn save(&self, key: &CredentialKey, secret: &CredentialSecret) -> Result<(), AuthError> {
        let payload = serde_json::to_string(secret)?;
        self.entry(key)?
            .set_password(&payload)
            .map_err(AuthError::Keyring)
    }

    fn get(&self, key: &CredentialKey) -> Result<Option<CredentialSecret>, AuthError> {
        match self.entry(key)?.get_password() {
            Ok(payload) => Ok(Some(serde_json::from_str(&payload)?)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(AuthError::Keyring(err)),
        }
    }

    fn delete(&self, key: &CredentialKey) -> Result<(), AuthError> {
        match self.entry(key)?.delete_password() {
            Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(AuthError::Keyring(err)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedToken {
    pub value: String,
    pub expires_at: Option<Instant>,
}

impl CachedToken {
    pub fn new(value: impl Into<String>, ttl: Option<Duration>) -> Self {
        Self {
            value: value.into(),
            expires_at: ttl.map(|duration| Instant::now() + duration),
        }
    }

    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expiry) => Instant::now() < expiry,
            None => true,
        }
    }
}

#[derive(Debug, Default)]
pub struct TokenCache {
    inner: Mutex<HashMap<CredentialKey, CachedToken>>,
}

impl TokenCache {
    pub fn insert(&self, key: CredentialKey, token: CachedToken) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.insert(key, token);
        }
    }

    pub fn get(&self, key: &CredentialKey) -> Option<String> {
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(entry) = inner.get(key) {
                if entry.is_valid() {
                    return Some(entry.value.clone());
                }
            }
            inner.remove(key);
        }
        None
    }

    pub fn clear(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.clear();
        }
    }
}

#[derive(Debug, Clone)]
pub struct RedfishSession {
    pub session_id: String,
    pub auth_token: String,
    pub expires_at: Option<Instant>,
}

impl RedfishSession {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expiry) => Instant::now() >= expiry,
            None => false,
        }
    }

    pub fn into_cached_token(self) -> CachedToken {
        CachedToken {
            value: self.auth_token,
            expires_at: self.expires_at,
        }
    }
}

#[derive(Clone)]
pub struct CredentialManager {
    store: Arc<dyn CredentialStore>,
    token_cache: Arc<TokenCache>,
    interactive: bool,
}

impl CredentialManager {
    pub fn new(store: Arc<dyn CredentialStore>) -> Self {
        Self {
            store,
            token_cache: Arc::new(TokenCache::default()),
            interactive: true,
        }
    }

    pub fn with_default_keyring() -> Self {
        let store: Arc<dyn CredentialStore> =
            Arc::new(KeyringCredentialStore::new("FabreXLens"));
        Self::new(store)
    }

    pub fn with_interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    pub fn ensure_credentials(&self, key: &CredentialKey) -> Result<CredentialSecret, AuthError> {
        if let Some(secret) = self.store.get(key)? {
            return Ok(secret);
        }

        if !self.interactive {
            return Err(AuthError::InteractiveDisabled(key.to_string()));
        }

        let secret = prompt_for_credentials(key)?;
        self.store.save(key, &secret)?;
        Ok(secret)
    }

    pub fn get_credentials(
        &self,
        key: &CredentialKey,
    ) -> Result<Option<CredentialSecret>, AuthError> {
        self.store.get(key)
    }

    pub fn set_credentials(
        &self,
        key: &CredentialKey,
        secret: &CredentialSecret,
    ) -> Result<(), AuthError> {
        self.store.save(key, secret)
    }

    pub fn delete_credentials(&self, key: &CredentialKey) -> Result<(), AuthError> {
        self.store.delete(key)
    }

    pub fn cache_token(&self, key: CredentialKey, token: CachedToken) {
        self.token_cache.insert(key, token);
    }

    pub fn cached_token(&self, key: &CredentialKey) -> Option<String> {
        self.token_cache.get(key)
    }

    pub fn clear_cache(&self) {
        self.token_cache.clear();
    }

    pub fn has_credentials(&self, key: &CredentialKey) -> Result<bool, AuthError> {
        self.store.get(key).map(|opt| opt.is_some())
    }

    pub fn auth_context(&self, key: &CredentialKey) -> Result<Option<AuthContext>, AuthError> {
        if let Some(token) = self.cached_token(key) {
            return Ok(Some(AuthContext::bearer(token)));
        }

        match self.get_credentials(key)? {
            Some(secret) => Ok(Some(secret.as_auth_context())),
            None => Ok(None),
        }
    }
}

pub fn prompt_for_credentials(key: &CredentialKey) -> Result<CredentialSecret, AuthError> {
    let theme = ColorfulTheme::default();

    let username: String = Input::with_theme(&theme)
        .with_prompt(format!("{} username", key))
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("username cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .map_err(AuthError::Prompt)?;

    let password = Password::with_theme(&theme)
        .with_prompt(format!("{} password", key))
        .allow_empty_password(false)
        .interact()
        .map_err(AuthError::Prompt)?;

    let api_token = Password::with_theme(&theme)
        .with_prompt(format!("{} API token (optional)", key))
        .allow_empty_password(true)
        .interact()
        .map_err(AuthError::Prompt)?;

    let api_token = api_token.trim().to_owned();

    Ok(CredentialSecret {
        username,
        password,
        api_token: if api_token.is_empty() {
            None
        } else {
            Some(api_token)
        },
    })
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("prompt error: {0}")]
    Prompt(#[from] dialoguer::Error),
    #[error("interactive prompts disabled; cannot create credentials for {0}")]
    InteractiveDisabled(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MemoryStore {
        data: Mutex<HashMap<String, CredentialSecret>>,
    }

    impl MemoryStore {
        fn new() -> Self {
            Self {
                data: Mutex::new(HashMap::new()),
            }
        }
    }

    impl CredentialStore for MemoryStore {
        fn save(&self, key: &CredentialKey, secret: &CredentialSecret) -> Result<(), AuthError> {
            let mut data = self.data.lock().unwrap();
            data.insert(key.storage_key(), secret.clone());
            Ok(())
        }

        fn get(&self, key: &CredentialKey) -> Result<Option<CredentialSecret>, AuthError> {
            let data = self.data.lock().unwrap();
            Ok(data.get(&key.storage_key()).cloned())
        }

        fn delete(&self, key: &CredentialKey) -> Result<(), AuthError> {
            let mut data = self.data.lock().unwrap();
            data.remove(&key.storage_key());
            Ok(())
        }
    }

    #[test]
    fn auth_context_returns_none_when_missing() {
        let manager = CredentialManager::new(Arc::new(MemoryStore::new()));
        let key = CredentialKey::default(CredentialDomain::FabreX);
        assert!(manager.auth_context(&key).unwrap().is_none());
    }

    #[test]
    fn auth_context_uses_api_token_when_available() {
        let manager = CredentialManager::new(Arc::new(MemoryStore::new()));
        let key = CredentialKey::default(CredentialDomain::FabreX);
        let secret = CredentialSecret {
            username: "user".into(),
            password: "pass".into(),
            api_token: Some("token-123".into()),
        };
        manager.set_credentials(&key, &secret).unwrap();

        let ctx = manager.auth_context(&key).unwrap().unwrap();
        assert_eq!(ctx.bearer_token.as_deref(), Some("token-123"));
        assert!(ctx.basic.is_none());
    }

    #[test]
    fn auth_context_falls_back_to_basic_auth() {
        let manager = CredentialManager::new(Arc::new(MemoryStore::new()));
        let key = CredentialKey::default(CredentialDomain::FabreX);
        let secret = CredentialSecret {
            username: "user".into(),
            password: "pass".into(),
            api_token: None,
        };
        manager.set_credentials(&key, &secret).unwrap();

        let ctx = manager.auth_context(&key).unwrap().unwrap();
        assert!(ctx.bearer_token.is_none());
        assert_eq!(ctx.basic.as_ref().unwrap().0, "user");
    }

    #[test]
    fn auth_context_prefers_cached_token() {
        let manager = CredentialManager::new(Arc::new(MemoryStore::new()));
        let key = CredentialKey::default(CredentialDomain::FabreX);
        let secret = CredentialSecret {
            username: "user".into(),
            password: "pass".into(),
            api_token: None,
        };
        manager.set_credentials(&key, &secret).unwrap();
        manager.cache_token(
            key.clone(),
            CachedToken::new("cached-token", Some(Duration::from_secs(60))),
        );

        let ctx = manager.auth_context(&key).unwrap().unwrap();
        assert_eq!(ctx.bearer_token.as_deref(), Some("cached-token"));
        assert!(ctx.basic.is_none());
    }
}

