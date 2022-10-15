mod persistence;
mod telegram;
mod twitter;
mod types;
mod util;

use crate::mime::{APPLICATION_OCTET_STREAM, TEXT_VCARD};
use crate::persistence::Persister;
use crate::telegram::downloader::TelegramDownloader;
use crate::telegram::fetcher::TelegramGenerator;
use crate::twitter::create_token;
use crate::twitter::poster::TwitterPoster;
use crate::twitter::uploader::TwitterUploader;
use crate::types::Cfg;
use crate::types::{Processor, Runnable, Source};
use mime_guess::mime;
use simple_logger::SimpleLogger;
use tokio::fs;

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

    let mut persister = Persister::new(&config.data_dir).await;
    log::info!("Last processed id: {}", persister.get_last_id());

    let client = telegram::create_client(&config).await?;
    let token = create_token(&config.twitter);

    let mut generator = TelegramGenerator::new(client.clone(), &config, persister.get_last_id());
    let mut downloader = TelegramDownloader::new(client.clone(), &config);
    let mut twitter_uploader = TwitterUploader::new(token.clone(), &config);
    let mut twitter_poster = TwitterPoster::new(token.clone());

    generator
        .drain_to(&mut downloader)
        .connect_to(&mut twitter_uploader)
        .connect_to(&mut twitter_poster)
        .sink_at(&mut persister);

    let _ = tokio::join!(
        generator.run(),
        downloader.run(),
        twitter_uploader.run(),
        twitter_poster.run(),
        persister.run()
    );

    log::info!("End processing");
    Ok(())
}
