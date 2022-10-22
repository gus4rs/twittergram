use crate::telegram::types::{TelegramClient, TelegramMessage, TelegramMessageIter};
use crate::{telegram, Cfg};
use async_trait::async_trait;
use grammers_client::client::messages::{InvocationError, MessageIter};
use grammers_client::types::{Chat, Media, Message};
use grammers_client::Client;
use grammers_session::PackedChat;
use std::path::Path;

pub struct GrammersIter {
    iter: MessageIter,
}
impl GrammersIter {
    fn new(iter: MessageIter) -> Self {
        GrammersIter { iter }
    }
}
#[async_trait]
impl TelegramMessageIter<GrammersMessage> for GrammersIter {
    async fn total(&mut self) -> Result<usize, InvocationError> {
        self.iter.total().await
    }

    async fn next(&mut self) -> Result<Option<GrammersMessage>, InvocationError> {
        self.iter.next().await.map(|m| m.map(GrammersMessage::new))
    }
}

pub struct GrammersMessage {
    msg: Message,
}
impl GrammersMessage {
    fn new(msg: Message) -> Self {
        GrammersMessage { msg }
    }
}
impl TelegramMessage for GrammersMessage {
    fn id(&self) -> i32 {
        self.msg.id()
    }

    fn text(&self) -> &str {
        self.msg.text()
    }

    fn grouped_id(&self) -> Option<i64> {
        self.msg.grouped_id()
    }

    fn media(&self) -> Option<Media> {
        self.msg.media()
    }
}

#[derive(Debug, Clone)]
pub struct GrammersClient {
    client: Client,
}

impl GrammersClient {
    pub async fn new(config: &Cfg) -> Self {
        GrammersClient {
            client: telegram::create_client(config).await.ok().unwrap(),
        }
    }
}

#[async_trait]
impl TelegramClient for GrammersClient {
    type M = GrammersMessage;
    type I = GrammersIter;

    async fn resolve_username(&self, username: &str) -> Result<Option<Chat>, InvocationError> {
        self.client.resolve_username(username).await
    }

    fn iter_messages<C: Into<PackedChat>>(&self, chat: C) -> GrammersIter {
        let buffer = self.client.iter_messages(chat);
        GrammersIter::new(buffer)
    }

    async fn download_media<P: AsRef<Path> + Send>(
        &self,
        media: &Media,
        path: P,
    ) -> Result<(), std::io::Error> {
        self.client.download_media(media, path).await
    }
}
