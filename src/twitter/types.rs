use async_trait::async_trait;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TwitterBuilder {
    media_ids: Vec<u64>,
    text: String,
}

impl TwitterBuilder {
    pub(crate) fn new() -> Self {
        TwitterBuilder {
            media_ids: vec![],
            text: "".to_string(),
        }
    }
    pub fn add_media(&mut self, media_id: u64) {
        let _ = &self.media_ids.push(media_id);
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn media_ids(&self) -> &Vec<u64> {
        &self.media_ids
    }
    pub fn text(&self) -> String {
        self.text.clone()
    }
}

#[async_trait]
pub trait Postable: Sync + Send + 'static {
    async fn upload_media(&mut self, file: &Path) -> Result<u64, Box<dyn Error>>;
    async fn send(&mut self) -> Result<String, Box<dyn Error + Send + Sync>>;
}

pub trait TwitterClient: Postable {
    fn new_builder(&mut self) -> &mut TwitterBuilder;
}
