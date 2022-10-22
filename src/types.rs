use crate::telegram::types::TelegramMessage;
use crate::{mime, APPLICATION_OCTET_STREAM, TEXT_VCARD};
use egg_mode::media::MediaId;
use grammers_client::types::Media;
use grammers_client::types::Media::{Contact, Document, Photo, Sticker};
use mime_guess::Mime;
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

#[derive(Deserialize, Debug)]
pub struct Cfg {
    pub(crate) data_dir: String,
    pub(crate) max_messages: i32,
    pub(crate) telegram: TelegramConfig,
    pub(crate) twitter: TwitterConfig,
}

#[derive(Deserialize, Debug)]
pub struct TelegramConfig {
    pub(crate) api_id: i32,
    pub(crate) api_hash: String,
    pub(crate) chat_name: String,
}

#[derive(Deserialize, Debug)]
pub struct TwitterConfig {
    pub(crate) api_key: String,
    pub(crate) api_secret: String,
    pub(crate) access_token: String,
    pub(crate) access_token_secret: String,
}

#[derive(Clone, Debug)]
pub struct Post {
    id: i32,
    text: String,
    tg_attachments: Vec<Attachment>,
    tw_attachments: Vec<MediaId>,
}

impl Post {
    pub fn new(id: i32, text: String) -> Post {
        Post {
            id,
            text,
            tg_attachments: vec![],
            tw_attachments: vec![],
        }
    }

    pub(crate) fn from_message<M: TelegramMessage>(msg: &M) -> Post {
        let mut post = Post::new(msg.id(), msg.text().to_string());
        if let Some(media) = msg.media() {
            post.add_tg_attachment(Attachment::new(media));
        }
        post
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn attachments(&self) -> &Vec<Attachment> {
        &self.tg_attachments
    }

    pub fn tw_attachments(&self) -> &Vec<MediaId> {
        &self.tw_attachments
    }

    fn get_suffix(mime: &Mime) -> String {
        let extension = mime_guess::get_mime_extensions(mime)
            .map(|o| o[0])
            .unwrap_or("");
        if extension.is_empty() {
            String::new()
        } else {
            String::from(".") + extension
        }
    }

    pub fn add_twitter_attachment(&mut self, attachment: MediaId) {
        self.tw_attachments.push(attachment);
    }

    pub fn add_tg_attachment(&mut self, mut attachment: Attachment) {
        let len = self.tg_attachments.len();
        let suffix = Post::get_suffix(&attachment.mime);
        attachment.path = format!("message-{}_{}{}", self.id(), len, suffix);
        self.tg_attachments.push(attachment)
    }

    pub fn validate(&self, ignore: &str) -> bool {
        let empty = self.text.is_empty() && self.tg_attachments.is_empty();
        !empty && !self.text.contains(ignore)
    }
}

#[derive(Clone, Debug)]
pub struct Attachment {
    tg_media: Media,
    mime: Mime,
    path: String,
}

impl Attachment {
    pub fn tg_media(&self) -> &Media {
        &self.tg_media
    }

    pub fn mime(&self) -> &Mime {
        &self.mime
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn new(media: Media) -> Attachment {
        let mime = Attachment::extract_media(&media);
        Attachment {
            tg_media: media,
            mime,
            path: String::new(),
        }
    }

    fn extract_media(media: &Media) -> Mime {
        match media {
            Photo(_) => mime::IMAGE_JPEG,
            Sticker(sticker) => Attachment::parse_mime(sticker.document.mime_type()),
            Document(document) => Attachment::parse_mime(document.mime_type()),
            Contact(_) => TEXT_VCARD,
            _ => APPLICATION_OCTET_STREAM,
        }
    }

    fn parse_mime(mime_type: Option<&str>) -> Mime {
        mime_type
            .map(|m| m.parse().unwrap())
            .unwrap_or(APPLICATION_OCTET_STREAM)
    }
}

pub trait Runnable {
    fn run(self) -> JoinHandle<()>;
}

pub trait Source<A>: Runnable {
    fn set_output(&mut self, output: Sender<A>);

    fn drain_to<'a, B, P: Processor<A, B>>(&mut self, processor: &'a mut P) -> &'a mut P {
        let (sender, receiver): (Sender<A>, Receiver<A>) = mpsc::channel(1000);
        self.set_output(sender);
        processor.set_input(receiver);
        processor
    }
}

pub trait Sink<X>: Runnable {
    fn set_input(&mut self, receiver: Receiver<X>);
}

pub trait Processor<A, B>: Runnable {
    fn set_input(&mut self, input: Receiver<A>);
    fn set_output(&mut self, output: Sender<B>);
    fn connect_to<'a, C, Q: Processor<B, C>>(&mut self, another: &'a mut Q) -> &'a mut Q {
        let (sender, receiver): (Sender<B>, Receiver<B>) = mpsc::channel(1000);
        self.set_output(sender);
        another.set_input(receiver);
        another
    }
    fn sink_at<S: Sink<B>>(&mut self, sink: &mut S) {
        let (sender, receiver): (Sender<B>, Receiver<B>) = mpsc::channel(1000);
        sink.set_input(receiver);
        self.set_output(sender);
    }
}
