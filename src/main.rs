use std::{fs::File, io::Read, path::PathBuf};

use dirs::config_dir;
use serde::Deserialize;
use teloxide_core::{
    payloads::setters::*,
    requests::{Request, Requester},
    types::{ChatId, InputFile, ParseMode},
    Bot,
};

#[derive(Deserialize)]
struct Config<'a> {
    token: &'a str,
    proxy: Option<&'a str>,
    master_chat_id: ChatId,
    prefix: Option<&'a str>,
}

fn default_cfg_path() -> PathBuf {
    let mut dir = config_dir()
        .expect("Error obtaining config directory. Specify path to config file with '-c'");
    dir.push("notify-tg.toml");
    dir
}

#[derive(argh::FromArgs)]
/// Send message to yourself in Telegram
struct Args {
    #[argh(option, short = 'c', default = "default_cfg_path()")]
    /// alternative path to config. Default is $sys_config_dir/notify-tg.toml
    cfg_path: PathBuf,
    /// file to send
    #[argh(option, short = 'i')]
    include: Option<String>,
    /// text message to send
    #[argh(positional)]
    message: Option<String>,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let Args {
        cfg_path,
        message,
        include,
    } = argh::from_env();

    let mut config_file = File::open(&cfg_path)
        .unwrap_or_else(|err| panic!("Error opening config {:?}: {:?}", cfg_path, err));

    let mut config_raw = String::new();
    config_file
        .read_to_string(&mut config_raw)
        .unwrap_or_else(|err| panic!("Error reading config: {:?}", err));

    let Config {
        token,
        proxy,
        master_chat_id,
        prefix,
    } = toml::from_str(&config_raw).unwrap_or_else(|err| panic!("Error parsing config: {:?}", err));

    let bot = Bot::with_client(
        token,
        match proxy {
            Some(proxy) => reqwest::Client::builder()
                .proxy(
                    reqwest::Proxy::https(proxy)
                        .unwrap_or_else(|err| panic!("Error creating reqwest::Proxy: {:?}", err)),
                )
                .build()
                .unwrap_or_else(|err| panic!("Error creating reqwest::Client: {:?}", err)),
            None => reqwest::Client::new(),
        },
    );

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("Error building tokio::runtime::Runtime");
    let res = rt.block_on(async move {
        match (message, include) {
            (Some(message), Some(include)) => {
                let message = prefix
                    .map_or_else(|| String::with_capacity(message.len()), str::to_owned)
                    + &message;
                bot.send_document(master_chat_id, InputFile::File(include.into()))
                    .caption(message)
                    .parse_mode(ParseMode::Html)
                    .send()
                    .await
                    .map(drop)
            }
            (Some(message), None) => {
                let message = prefix
                    .map_or_else(|| String::with_capacity(message.len()), str::to_owned)
                    + &message;
                bot.send_message(master_chat_id, message)
                    .parse_mode(ParseMode::Html)
                    .send()
                    .await
                    .map(drop)
            }
            (None, Some(include)) => {
                let res = bot.send_document(master_chat_id, InputFile::File(include.into()));
                match prefix {
                    Some(prefix) => res.caption(prefix),
                    None => res,
                }
                .parse_mode(ParseMode::Html)
                .send()
                .await
                .map(drop)
            }
            (None, None) => bot.get_me().send().await.map(|me| {
                log::info!("getMe -> {:#?}", me);
                log::info!("Config is fine. Exiting.");
                log::info!("For help use `notify-tg --help`");
            }),
        }
    });
    if let Err(err) = res {
        log::error!("{:?}", err);
    }
}
