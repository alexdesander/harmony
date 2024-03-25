use random_string::generate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ApiToken(String);

impl ApiToken {
    pub fn new() -> Self {
        let charset = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        Self(generate(20, charset))
    }

    pub fn from_string(token: &str) -> Self {
        Self(token.to_string())
    }

    pub fn as_str<'a>(&'a self) -> &'a str {
        &self.0
    }
}
