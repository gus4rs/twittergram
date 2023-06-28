use crate::twitter::types::{Postable, TwitterBuilder, TwitterClient};
use crate::types::Cfg;
use async_trait::async_trait;
use critter::auth::TwitterAuth;
use critter::TwitterClient as Critter;
use std::error::Error;
use std::path::Path;

#[derive(Clone)]
pub struct CritterClient {
    builder: TwitterBuilder,
    client: Critter,
}

impl CritterClient {
    pub fn new(config: &Cfg) -> Self {
        let auth = TwitterAuth::from_oa1uc(
            &config.twitter.api_key,
            &config.twitter.api_secret,
            &config.twitter.access_token,
            &config.twitter.access_token_secret,
        );
        let cli = match Critter::new(auth) {
            Ok(c) => c,
            Err(err) => panic!("Error creating Twitter client {}", err),
        };
        CritterClient {
            builder: TwitterBuilder::new(),
            client: cli,
        }
    }
}

#[async_trait]
impl Postable for CritterClient {
    async fn upload_media(&mut self, file: &Path) -> Result<u64, Box<dyn Error>> {
        let filename = file
            .file_name()
            .map(|o| o.to_os_string().into_string().unwrap());
        let path = file.to_str().expect("Error getting file path");
        match self.client.upload_media(path, filename).await {
            Ok(mut res) => Ok(res.id().parse().unwrap()),
            Err(err) => Err(Box::try_from(err).unwrap()),
        }
    }

    async fn send(&mut self) -> Result<String, Box<dyn Error + Send + Sync>> {
        match self
            .client
            .tweet(|tweet| {
                let x = tweet.text(self.builder.text().as_str());
                if !self.builder.media_ids().is_empty() {
                    x.media(|mb| {
                        for id in self.builder.media_ids() {
                            mb.id(*id);
                        }
                        mb
                    });
                }
                x
            })
            .await
        {
            Ok(response) => Ok(response.id().to_string()),
            Err(e) => Err(Box::try_from(e).unwrap()),
        }
    }
}

impl TwitterClient for CritterClient {
    fn new_builder(&mut self) -> &mut TwitterBuilder {
        self.builder = TwitterBuilder::new();
        &mut self.builder
    }
}
