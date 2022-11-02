use crate::twitter::types::TwitterClient;
use crate::types::{Post, Processor, Runnable};
use egg_mode::error::Error;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

pub struct TwitterPoster<C: TwitterClient> {
    client: C,
    sender: Option<Sender<Post>>,
    receiver: Option<Receiver<Post>>,
}

impl<C: TwitterClient> TwitterPoster<C> {
    pub fn new(client: C) -> Self {
        TwitterPoster {
            client,
            receiver: None,
            sender: None,
        }
    }
}

impl<C: TwitterClient> Processor<Post, Post> for TwitterPoster<C> {
    fn set_input(&mut self, input: Receiver<Post>) {
        self.receiver = Some(input);
    }
    fn set_output(&mut self, output: Sender<Post>) {
        self.sender = Some(output);
    }
}

impl<C: TwitterClient> Runnable for TwitterPoster<C> {
    fn run(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match self.receiver.as_mut().unwrap().recv().await {
                    None => {
                        break;
                    }
                    Some(msg) => {
                        let text = msg.text().to_string();
                        let builder = self.client.new_builder();
                        builder.set_text(text);

                        for attachment in msg.tw_attachments() {
                            builder.add_media(attachment.clone());
                        }

                        if builder.text().is_empty() && builder.media_ids().is_empty() {
                            log::info!("Ignored telegram post {} with no text and media", msg.id());
                        } else {
                            match self.client.send().await {
                                Ok(_) => {
                                    let id = msg.id();
                                    self.sender.as_ref().unwrap().send(msg).await.expect("TODO");
                                    log::info!("Successfully posted tweet for {}", id);
                                }
                                Err(Error::TwitterError(_, twitter_errors))
                                    if twitter_errors.errors.len() == 1 =>
                                {
                                    if let Some(code) = twitter_errors.errors.first() {
                                        if code.code == 324 {
                                            log::warn!("Error sending tweet {}, the media is unsupported. This will not be retried: {:?}", msg.id(), code);
                                        }
                                        self.sender
                                            .as_ref()
                                            .unwrap()
                                            .send(msg)
                                            .await
                                            .expect("send");
                                    }
                                }
                                Err(e) => {
                                    panic!("[Poster] Error sending Tweet for {}:{:?}", msg.id(), e);
                                }
                            }
                        }
                    }
                }
            }
        })
    }
}
