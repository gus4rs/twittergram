# Twittergram

<img alt="Twittergram" height="256" src="images/blue.png" width="256"/>

Twittergram is a simple utility to mirror a telegram public chat to a Twitter account:

* Supports Telegram messages with images and videos
* Has basic support for Telegram albums (posts with multiple media)
* Can ignore some telegram posts by adding a special ```#tgonly``` keyword to messages 
* Uses the only [pure Rust Telegram client](https://github.com/Lonami/grammers)

## How to use it

Copy [config.toml.example](config.toml.example) to ```config.toml``` and adjust the configuration accordingly

then run `./twittergram`

## Examples

### Keeping Telegram and Twitter in sync

In the ```config.toml``` file:

```toml
# Maximum number of messages to retrieve from Telegram
max_messages=1
```

Then schedule ```twittergram``` to run periodically (e.g. [systemd timer](https://opensource.com/article/20/7/systemd-timers)) every 1 minute 

## Installation

```bash
$ cargo build -r
```

The binary ```twittergram``` for your platform will be at ```target/release```