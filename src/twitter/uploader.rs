use crate::twitter::types::TwitterClient;
use crate::types::{Post, Processor, Runnable};
use crate::Cfg;
use log::warn;
use std::path::PathBuf;
use critter::error::Error;
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
                            let media_type = attachment.mime();
                            let result = self.client.upload_media(buf.as_path(), media_type).await;

                            match result {
                                Ok(id) => {
                                    log::info!(
                                        "Media {} successfully processed",
                                        attachment.path()
                                    );
                                    media_ids.push(id);
                                }
                                Err(err) => {
                                    warn!(
                                        "[Uploader] Error uploading media {} : {:?}",
                                        msg.id(),
                                        err
                                    );
                                    if let Some(bad_media) = err.downcast_ref::<Error>() {
                                        println!("The media is not supported by Twitter, this will no be retried {}", bad_media);
                                    } else {
                                        attach_failed = true;
                                    }
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
