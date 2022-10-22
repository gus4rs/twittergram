use std::path::PathBuf;

use crate::telegram::types::TelegramClient;
use crate::types::{Attachment, Cfg};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

use crate::types::Post;
use crate::types::{Processor, Runnable};

pub struct TelegramDownloader<T: TelegramClient> {
    client: T,
    receiver: Option<Receiver<Post>>,
    sender: Option<Sender<Post>>,
    path: String,
}

impl<T: TelegramClient> TelegramDownloader<T> {
    pub(crate) fn new(client: T, cfg: &Cfg) -> Self {
        TelegramDownloader {
            client,
            receiver: None,
            sender: None,
            path: cfg.data_dir.clone(),
        }
    }

    pub fn get_save_path(&self, attachment: &Attachment, msg_id: i32, index: usize) -> PathBuf {
        let attachment_mime = &attachment.mime();
        let extension = mime_guess::get_mime_extensions(attachment_mime)
            .map(|o| o[0])
            .unwrap_or("");
        let suffix = if extension.is_empty() {
            "".to_string()
        } else {
            ".".to_owned() + extension
        };
        let filename = format!("message-{}_{}{}", msg_id, index, suffix);
        let mut buf = PathBuf::from(&self.path);
        buf.push(filename);
        buf
    }
}

impl<T: TelegramClient> Processor<Post, Post> for TelegramDownloader<T> {
    fn set_input(&mut self, input: Receiver<Post>) {
        self.receiver = Some(input);
    }
    fn set_output(&mut self, output: Sender<Post>) {
        self.sender = Some(output);
    }
}

impl<T: TelegramClient> Runnable for TelegramDownloader<T> {
    fn run(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match self.receiver.as_mut().unwrap().recv().await {
                    None => {
                        break;
                    }
                    Some(msg) => {
                        log::info!("Received {:?}", msg.id());
                        for i in 0..msg.attachments().len() {
                            let attachment = &msg.attachments()[i];
                            let path = self.get_save_path(attachment, msg.id(), i);

                            self.client
                                .download_media(attachment.tg_media(), path.as_path())
                                .await
                                .expect("Error downloading message");
                        }
                        self.sender
                            .as_ref()
                            .unwrap()
                            .send(msg.clone())
                            .await
                            .expect("TODO: panic message");
                    }
                }
            }
        })
    }
}
