use async_trait::async_trait;
use std::io::Error;
use std::path::Path;

use grammers_client::client::auth::InvocationError;
use grammers_client::types::{Chat, Media};
use grammers_session::PackedChat;

#[async_trait]
pub trait TelegramClient: Sync + Send + 'static {
    type M: TelegramMessage;
    type I: TelegramMessageIter<Self::M>;
    async fn resolve_username(&self, username: &str) -> Result<Option<Chat>, InvocationError>;
    fn iter_messages<C: Into<PackedChat>>(&self, chat: C) -> Self::I;
    async fn download_media<P: AsRef<Path> + Send>(
        &self,
        media: &Media,
        path: P,
    ) -> Result<(), Error>;
}

#[async_trait]
pub trait TelegramMessageIter<M: TelegramMessage>: Send + Sync {
    async fn total(&mut self) -> Result<usize, InvocationError>;
    async fn next(&mut self) -> Result<Option<M>, InvocationError>;
}

pub trait TelegramMessage: Send {
    fn id(&self) -> i32;
    fn text(&self) -> &str;
    fn grouped_id(&self) -> Option<i64>;
    fn media(&self) -> Option<Media>;
}
