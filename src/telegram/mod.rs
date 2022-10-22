pub(crate) mod downloader;
pub(crate) mod fetcher;
pub(crate) mod telegram_client;
pub(crate) mod types;

use std::collections::{BTreeMap, HashMap};
use std::io::stdin;
use std::path::PathBuf;

use crate::util::read_input;
use crate::Cfg;
use grammers_client::client::auth::InvocationError;
use grammers_client::{Client, Config, InitParams};
use grammers_session::Session;

static SESSION_NAME: &str = "telegram.session";

async fn create_client(config: &Cfg) -> Result<Client, InvocationError> {
    let mut path_buf = PathBuf::from(&config.data_dir);
    path_buf.push(SESSION_NAME);
    let cfg = Config {
        session: Session::load_file_or_create(path_buf.as_path())
            .expect("Cannot initialize session"),
        api_id: config.telegram.api_id,
        api_hash: config.telegram.api_hash.clone(),
        params: InitParams {
            catch_up: true,
            ..Default::default()
        },
    };

    let future = Client::connect(cfg).await;

    let mut client = future.ok().unwrap();

    match client.is_authorized().await {
        Ok(false) => {
            handle(
                &mut client,
                config.telegram.api_id,
                config.telegram.api_hash.clone(),
                path_buf,
            )
            .await
        }
        Ok(true) => log::info!("Telegram client crerated"),
        Err(e) => {
            panic!("{:?}", e);
        }
    }

    Ok(client)
}

//Handles Telegram session establishment
async fn handle(cli: &mut Client, api_id: i32, api_hash: String, path_buf: PathBuf) {
    let phone: String = read_input("Telephone number".to_string(), &mut stdin().lock());

    match cli
        .request_login_code(phone.as_str(), api_id, &api_hash)
        .await
    {
        Ok(token) => {
            let read_token: String = read_input(
                "Check the Telegram App and type the Token".to_string(),
                &mut stdin().lock(),
            );
            match cli.sign_in(&token, read_token.as_str()).await {
                Ok(user) => {
                    log::info!("Signed in!, {}", user.first_name());
                    cli.session()
                        .save_to_file(path_buf.as_path())
                        .expect("Error saving");
                }
                Err(e) => {
                    panic!("{:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Error getting token {:?}", e)
        }
    }
}

pub async fn _message_stats(
    client: &mut Client,
    chat_name: String,
) -> Result<BTreeMap<i32, String>, InvocationError> {
    let mut contacts: HashMap<String, i32> = HashMap::new();

    let maybe_chat = client.resolve_username(chat_name.as_str()).await?;

    let chat = maybe_chat.unwrap_or_else(|| panic!("Chat {} could not be found", chat_name));

    let mut messages = client.iter_messages(&chat);

    while let Some(msg) = messages.next().await? {
        let name = msg
            .sender()
            .map(|c| c.name().to_string())
            .unwrap_or_else(|| "Nobody".to_string());
        *contacts.entry(name).or_insert(0) += 1;
    }

    let result: BTreeMap<i32, String> = contacts.into_iter().map(|(k, v)| (v, k)).collect();

    Ok(result)
}
