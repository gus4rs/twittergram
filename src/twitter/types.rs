use crate::mime::Mime;
use async_trait::async_trait;
use egg_mode::error::Error;
use egg_mode::media::{MediaHandle, MediaId};
use egg_mode::tweet::{DraftTweet, Tweet};
use egg_mode::{error, Response};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct TwitterBuilder {
    pub tweet: DraftTweet,
}

impl TwitterBuilder {
    pub(crate) fn new() -> Self {
        TwitterBuilder {
            tweet: DraftTweet::new(String::new()),
        }
    }
    pub fn add_media(&mut self, media_id: MediaId) {
        let _ = &self.tweet.add_media(media_id);
    }

    pub fn set_text(&mut self, text: String) {
        self.tweet.text = Cow::from(text);
    }

    pub fn media_ids(&self) -> &Vec<MediaId> {
        &self.tweet.media_ids
    }
    pub fn text(&self) -> String {
        self.tweet.text.to_string()
    }
}

#[async_trait]
pub trait Postable: Sync + Send + 'static {
    async fn upload_media(&mut self, data: &[u8], media_type: &Mime) -> error::Result<MediaHandle>;
    async fn send(&mut self) -> Result<Response<Tweet>, Error>;
    async fn get_status(&self, media_id: MediaId) -> error::Result<MediaHandle>;
}

pub trait TwitterClient: Postable {
    fn new_builder(&mut self) -> &mut TwitterBuilder;
}
