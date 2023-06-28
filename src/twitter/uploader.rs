use crate::twitter::types::TwitterClient;
use crate::types::{Post, Processor, Runnable};
use crate::Cfg;
use log::warn;
use std::path::PathBuf;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

pub struct TwitterUploader<C> {
    client: C,
    data_dir: String,
    receiver: Option<Receiver<Post>>,
    sender: Option<Sender<Post>>,
}

impl<C: TwitterClient> TwitterUploader<C> {
    pub fn new(client: C, cfg: &Cfg) -> Self {
        TwitterUploader {
            client,
            data_dir: cfg.data_dir.clone(),
            receiver: None,
            sender: None,
        }
    }
}

impl<C: TwitterClient> Processor<Post, Post> for TwitterUploader<C> {
    fn set_input(&mut self, input: Receiver<Post>) {
        self.receiver = Some(input);
    }
    fn set_output(&mut self, output: Sender<Post>) {
        self.sender = Some(output);
    }
}

impl<C: TwitterClient> Runnable for TwitterUploader<C> {
    fn run(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match self.receiver.as_mut().unwrap().recv().await {
                    None => {
                        break;
                    }
                    Some(mut msg) => {
                        let mut attach_failed = false;
                        let mut media_ids = vec![];
                        for attachment in msg.attachments() {
                            let mut buf = PathBuf::from(&self.data_dir);
                            buf.push(attachment.path());

                            let result = self.client.upload_media(buf.as_path()).await;

                            match result {
                                Ok(id) => {
                                    log::info!(
                                        "Media {} successfully processed",
                                        attachment.path()
                                    );
                                    media_ids.push(id);
                                }
                                Err(err) => {
                                    attach_failed = true;
                                    warn!(
                                        "[Uploader] Error uploading media {} : {:?}",
                                        msg.id(),
                                        err
                                    );
                                }
                            }
                        }
                        if !attach_failed {
                            for media in media_ids {
                                msg.add_twitter_attachment(media);
                            }
                            self.sender.as_ref().unwrap().send(msg).await.expect("TODO");
                        }
                    }
                }
            }
        })
    }
}
