use crate::types::Post;
use crate::types::{Runnable, Sink};
use serde::{Deserialize, Serialize};
use std::io::{ErrorKind, SeekFrom};
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;

pub struct Persister {
    state_file: File,
    state: State,
    receiver: Option<Receiver<Post>>,
}

const STATE_FILE: &str = "state";

#[derive(Serialize, Deserialize)]
struct State {
    tg_id: i32,
}

impl Persister {
    pub async fn new(data_file: &String) -> Persister {
        Persister::check_data_dir(data_file).await;

        let mut path = PathBuf::from(data_file);
        path.push(STATE_FILE);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(false)
            .open(path.as_path())
            .await
            .expect("Error opening file");

        let mut content: String = String::new();
        let s = file.read_to_string(&mut content).await.expect("Read file");
        let state: State = if s == 0 {
            State { tg_id: -1 }
        } else {
            serde_json::from_str(&content).unwrap()
        };
        Persister {
            state_file: file,
            state,
            receiver: None,
        }
    }

    pub fn get_last_id(&self) -> i32 {
        self.state.tg_id
    }

    async fn check_data_dir(name: &str) {
        match fs::metadata(name).await {
            Ok(m) => {
                if m.is_dir() {
                    log::info!("Data dir is {}", name);
                } else {
                    panic!("{} is not a directory", name)
                }
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                fs::create_dir(name).await.expect("Error creating data dir");
                log::info!("Create data dir {}", name);
            }
            Err(e) => panic!("{}", e),
        }
    }
    async fn save_state(&mut self) {
        let string = serde_json::to_string(&self.state).expect("Save to file");
        self.state_file
            .seek(SeekFrom::Start(0))
            .await
            .expect("Seek File");
        self.state_file
            .write_all((string).as_ref())
            .await
            .expect("Save file");
        self.state_file
            .set_len(string.len() as u64)
            .await
            .expect("truncate");
    }
}

impl Sink<Post> for Persister {
    fn set_input(&mut self, receiver: Receiver<Post>) {
        self.receiver = Some(receiver);
    }
}

impl Runnable for Persister {
    fn run(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match self.receiver.as_mut().unwrap().recv().await {
                    None => {
                        break;
                    }
                    Some(post) => {
                        self.state.tg_id = post.id();
                        self.save_state().await;
                        log::info!("Saved {:?}", self.state.tg_id);
                    }
                }
            }
        })
    }
}
