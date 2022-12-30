use crate::twitter::types::TwitterClient;
use crate::types::{Post, Processor, Runnable};
use crate::Cfg;
use egg_mode::media::ProgressInfo;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
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
                            let mut contents = vec![];

                            let mut buf = PathBuf::from(&self.data_dir);
                            buf.push(attachment.path());
                            let mut file = File::open(buf.as_path()).await.expect("File missing");
                            file.read_to_end(&mut contents)
                                .await
                                .expect("Error reading file");

                            let handle = self
                                .client
                                .upload_media(contents.as_slice(), attachment.mime())
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
                                        match self.client.get_status(handle.id.clone()).await {
                                            Ok(p) => progress = p.progress,
                                            Err(err) => {
                                                log::info!(
                                                    "Media format not supported {} : {:?}",
                                                    msg.id(),
                                                    err
                                                );
                                                break;
                                            }
                                        }
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
