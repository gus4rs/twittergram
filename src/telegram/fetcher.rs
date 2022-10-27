use crate::telegram::types::{TelegramClient, TelegramMessage, TelegramMessageIter};
use crate::types::{Attachment, Post, Runnable, Source};
use crate::Cfg;
use std::collections::vec_deque::{Iter, VecDeque};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

const IGNORE: &str = "#tgonly";

pub struct TelegramGenerator<T: TelegramClient> {
    client: T,
    chat_name: String,
    last_id: i32,
    size: i32,
    sender: Option<Sender<Post>>,
}

impl<T: TelegramClient> TelegramGenerator<T> {
    pub(crate) fn new(client: T, config: &Cfg, last_id: i32) -> Self {
        TelegramGenerator {
            client,
            chat_name: config.telegram.chat_name.clone(),
            last_id,
            size: config.max_messages,
            sender: None,
        }
    }
}

impl<T: TelegramClient> Source<Post> for TelegramGenerator<T> {
    fn set_output(&mut self, output: Sender<Post>) {
        self.sender = Some(output);
    }
}

struct FixedDeque<T> {
    queue: VecDeque<T>,
    size: i32,
}

impl<T> FixedDeque<T> {
    fn iterator(&mut self) -> Iter<'_, T> {
        self.queue.iter()
    }
    fn new(size: i32) -> Self {
        FixedDeque {
            queue: VecDeque::new(),
            size,
        }
    }

    fn push(&mut self, element: T) {
        self.queue.push_front(element);
        if self.queue.len() > self.size as usize {
            self.queue.pop_back();
        }
    }
}

struct Album<M: TelegramMessage> {
    items: Vec<M>,
    id: Option<i64>,
}

impl<M: TelegramMessage> Album<M> {
    fn new() -> Self {
        Album {
            items: vec![],
            id: None,
        }
    }

    fn start(&mut self, id: Option<i64>) {
        self.id = id;
    }

    fn close(&mut self) -> Post {
        let text = self
            .items
            .iter()
            .find(|m| !m.text().is_empty())
            .map(|m| m.text())
            .unwrap_or("");
        let mut post = Post::new(self.items.last().unwrap().id(), text.to_string());
        self.items
            .iter()
            .rev()
            .for_each(|m| post.add_tg_attachment(Attachment::new(m.media().unwrap())));
        self.items.clear();
        self.id = None;
        post
    }

    fn get_group(&self) -> i64 {
        self.id.unwrap()
    }

    fn get_msg_id(&self) -> i32 {
        self.items.last().unwrap().id()
    }

    fn add_item(&mut self, m: M) {
        self.items.push(m);
    }
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T: TelegramClient> Runnable for TelegramGenerator<T> {
    fn run(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            let chat = match self.client.resolve_username(&*self.chat_name).await {
                Ok(Some(c)) => c,
                _ => {
                    panic!("Chat {} could not be found", &*self.chat_name);
                }
            };
            let mut messages = self.client.iter_messages(&chat);
            let mut album: Album<_> = Album::new();
            let mut temp_messages = FixedDeque::new(self.size);

            loop {
                match messages.next().await {
                    Ok(Some(msg)) => {
                        if msg.id() <= self.last_id
                            && (album.is_empty() || album.get_msg_id() <= self.last_id)
                        {
                            break;
                        }

                        match msg.grouped_id() {
                            None if album.is_empty() => {
                                // No opened album, simply post the message
                                let post = Post::from_message(&msg);

                                if post.validate(IGNORE) && msg.id() > self.last_id {
                                    temp_messages.push(post);
                                }
                            }
                            None => {
                                // Assumes album is finished, close it and post ir
                                // TODO: support interleaving of different albums and single messages
                                let album_post = album.close();
                                if album_post.validate(IGNORE) {
                                    temp_messages.push(album_post);
                                }

                                // Post current message
                                let post = Post::from_message(&msg);

                                if post.validate(IGNORE) && msg.id() > self.last_id {
                                    temp_messages.push(post);
                                }
                            }
                            Some(_) if album.is_empty() => {
                                // Message is part of an album, start tracking it
                                album.start(msg.grouped_id());
                                // Todo move to start
                                album.add_item(msg);
                            }
                            Some(g) if g == album.get_group() => {
                                // Message is part of a already tracked album
                                album.add_item(msg);
                            }
                            Some(_) => {
                                // Message is part of a different album; close current and start new
                                let album_post = album.close();
                                if album_post.validate(IGNORE) {
                                    temp_messages.push(album_post);
                                }

                                album.start(msg.grouped_id());
                                album.add_item(msg);
                            }
                        };
                    }
                    Ok(None) => break,
                    Err(e) => {
                        panic!("{}", e)
                    }
                }
            }

            for message in temp_messages.iterator() {
                log::info!("Emitting telegram post {:?}", message);
                match self.sender.as_ref().unwrap().send(message.clone()).await {
                    Ok(_) => {}
                    Err(e) => {
                        panic!("{}", e)
                    }
                }
            }
        })
    }
}
