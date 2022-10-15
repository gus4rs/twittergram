use crate::types::{Post, Processor, Runnable};
use egg_mode::tweet::DraftTweet;
use egg_mode::Token;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

pub struct TwitterPoster {
    token: Token,
    sender: Option<Sender<Post>>,
    receiver: Option<Receiver<Post>>,
}

impl TwitterPoster {
    pub fn new(token: Token) -> Self {
        TwitterPoster {
            token,
            receiver: None,
            sender: None,
        }
    }
}

impl Processor<Post, Post> for TwitterPoster {
    fn set_input(&mut self, input: Receiver<Post>) {
        self.receiver = Some(input);
    }
    fn set_output(&mut self, output: Sender<Post>) {
        self.sender = Some(output);
    }
}

impl Runnable for TwitterPoster {
    fn run(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match self.receiver.as_mut().unwrap().recv().await {
                    None => {
                        break;
                    }
                    Some(msg) => {
                        let text = msg.text().to_string();
                        let mut tweet = DraftTweet::new(text);

                        for attachment in msg.tw_attachments() {
                            tweet.add_media(attachment.clone());
                        }

                        if tweet.text.is_empty() && tweet.media_ids.is_empty() {
                            log::info!("Ignored telegram post {} with no text and media", msg.id());
                        } else {
                            match tweet.send(&self.token).await {
                                Ok(_) => {
                                    let id = msg.id();
                                    self.sender.as_ref().unwrap().send(msg).await.expect("TODO");
                                    log::info!("Successfully posted tweet for {}", id);
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
