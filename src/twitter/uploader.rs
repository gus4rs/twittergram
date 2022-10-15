use crate::types::{Post, Processor, Runnable};
use egg_mode::media::{get_status, upload_media, ProgressInfo};
use egg_mode::Token;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use crate::Cfg;

pub struct TwitterUploader {
    token: Token,
    data_dir: String,
    receiver: Option<Receiver<Post>>,
    sender: Option<Sender<Post>>,
}

impl TwitterUploader {
    pub fn new(token: Token, cfg: &Cfg) -> Self {
        TwitterUploader {
            token,
            data_dir: cfg.data_dir.clone(),
            receiver: None,
            sender: None,
        }
    }
}

impl Processor<Post, Post> for TwitterUploader {
    fn set_input(&mut self, input: Receiver<Post>) {
        self.receiver = Some(input);
    }
    fn set_output(&mut self, output: Sender<Post>) {
        self.sender = Some(output);
    }
}

impl Runnable for TwitterUploader {
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
                            let mut contents = vec![];

                            let mut buf = PathBuf::from(&self.data_dir);
                            buf.push(attachment.path());
                            let mut file = File::open(buf.as_path())
                                .await.expect("File missing");
                            file.read_to_end(&mut contents)
                                .await
                                .expect("Error reading file");

                            let handle =
                                upload_media(contents.as_slice(), attachment.mime(), &self.token)
                                    .await
                                    .expect("Error uploading media");
                            let mut progress = handle.progress;

                            loop {
                                match progress {
                                    None | Some(ProgressInfo::Success) => {
                                        log::info!(
                                            "Media {} successfully processed",
                                            attachment.path()
                                        );
                                        media_ids.push(handle.id);
                                        break;
                                    }
                                    Some(ProgressInfo::Pending(secs))
                                    | Some(ProgressInfo::InProgress(secs)) => {
                                        tokio::time::sleep(Duration::from_secs(secs)).await;
                                    }
                                    Some(ProgressInfo::Failed(err)) if err.code == 3 => {
                                        attach_failed = true;
                                        log::info!(
                                            "Media format not supported {} : {:?}",
                                            msg.id(),
                                            err
                                        );
                                        break;
                                    }
                                    Some(ProgressInfo::Failed(err)) => {
                                        panic!(
                                            "[Uploader] Error uploading media {} : {:?}",
                                            msg.id(),
                                            err
                                        );
                                    }
                                }
                                progress = get_status(handle.id.clone(), &self.token)
                                    .await
                                    .expect("Error progress")
                                    .progress;
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
