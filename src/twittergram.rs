use crate::persistence::Persister;
use crate::telegram::downloader::TelegramDownloader;
use crate::telegram::fetcher::TelegramGenerator;
use crate::telegram::types::TelegramClient;
use crate::twitter::poster::TwitterPoster;
use crate::twitter::types::TwitterClient;
use crate::twitter::uploader::TwitterUploader;
use crate::types::Cfg;
use crate::types::{Processor, Runnable, Source};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Twittergram<T, U> {
    config: Cfg,
    tg_client: U,
    tw_client: T,
}

impl<T: TwitterClient + Clone, U: TelegramClient + Clone> Twittergram<T, U> {
    pub fn new(config: Cfg, tg_client: U, tw_client: T) -> Self {
        Twittergram {
            config,
            tg_client,
            tw_client,
        }
    }

    pub async fn run(self) -> Result<()> {
        let mut persister = Persister::new(&self.config.data_dir).await;
        log::info!("Last processed id: {}", persister.get_last_id());

        let mut generator = TelegramGenerator::new(
            self.tg_client.clone(),
            &self.config,
            persister.get_last_id(),
        );
        let mut downloader = TelegramDownloader::new(self.tg_client.clone(), &self.config);
        let mut twitter_uploader = TwitterUploader::new(self.tw_client.clone(), &self.config);
        let mut twitter_poster = TwitterPoster::new(self.tw_client.clone());

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
}
