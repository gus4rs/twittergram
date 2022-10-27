extern crate core;

use mime_guess::mime;
use simple_logger::SimpleLogger;
use tokio::fs;

use crate::mime::{APPLICATION_OCTET_STREAM, TEXT_VCARD};
use crate::persistence::Persister;
use crate::telegram::telegram_client::GrammersClient;
use crate::twitter::egg_mode_client::EggTwitterClient;
use crate::twittergram::Twittergram;
use crate::types::Cfg;

mod persistence;
mod telegram;
mod twitter;
mod twittergram;
mod types;
mod util;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .with_utc_timestamps()
        .init()
        .unwrap();

    let config_file = fs::read_to_string("config.toml")
        .await
        .expect("Could not find file config.toml");

    let config: Cfg = toml::from_str(&config_file).unwrap();

    Persister::check_data_dir(&config.data_dir).await;

    let telegram_client = GrammersClient::new(&config).await;
    let twitter_client = EggTwitterClient::new(&config);

    Twittergram::new(config, telegram_client, twitter_client)
        .run()
        .await
}
