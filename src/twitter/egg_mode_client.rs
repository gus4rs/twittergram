use crate::mime::Mime;
use crate::twitter::types::{Postable, TwitterBuilder, TwitterClient};
use crate::types::TwitterConfig;
use crate::Cfg;
use async_trait::async_trait;
use egg_mode::error::Error;
use egg_mode::media::{get_status, upload_media, MediaHandle, MediaId};
use egg_mode::tweet::Tweet;
use egg_mode::{error, Response, Token};

#[derive(Debug, Clone)]
pub struct EggTwitterClient {
    builder: TwitterBuilder,
    token: Token,
}

impl EggTwitterClient {
    pub fn new(config: &Cfg) -> Self {
        EggTwitterClient {
            builder: TwitterBuilder::new(),
            token: create_token(&config.twitter),
        }
    }
}

impl TwitterClient for EggTwitterClient {
    fn new_builder(&mut self) -> &mut TwitterBuilder {
        self.builder = TwitterBuilder::new();
        &mut self.builder
    }
}

#[async_trait]
impl Postable for EggTwitterClient {
    async fn upload_media(&mut self, data: &[u8], media_type: &Mime) -> error::Result<MediaHandle> {
        upload_media(data, media_type, &self.token).await
    }

    async fn send(&mut self) -> Result<Response<Tweet>, Error> {
        self.builder.tweet.send(&self.token).await
    }

    async fn get_status(&self, media_id: MediaId) -> error::Result<MediaHandle> {
        get_status(media_id, &self.token).await
    }
}

pub fn create_token(cfg: &TwitterConfig) -> Token {
    let con_token = egg_mode::KeyPair::new(cfg.api_key.clone(), cfg.api_secret.clone());
    let access_token =
        egg_mode::KeyPair::new(cfg.access_token.clone(), cfg.access_token_secret.clone());
    Token::Access {
        consumer: con_token,
        access: access_token,
    }
}
