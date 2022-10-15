use crate::types::TwitterConfig;
use egg_mode::Token;

pub(crate) mod poster;
pub(crate) mod uploader;

pub fn create_token(cfg: &TwitterConfig) -> Token {
    let con_token = egg_mode::KeyPair::new(cfg.api_key.clone(), cfg.api_secret.clone());
    let access_token =
        egg_mode::KeyPair::new(cfg.access_token.clone(), cfg.access_token_secret.clone());
    Token::Access {
        consumer: con_token,
        access: access_token,
    }
}
