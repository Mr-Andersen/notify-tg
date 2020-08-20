use std::{fs::File, io::Read};

use dirs::config_dir;
use serde::Deserialize;
use structopt::StructOpt;
use teloxide_core::{
    requests::Request,
    types::{ChatId, InputFile, ParseMode},
    BotBuilder,
};

#[derive(Deserialize)]
struct Config<'a> {
    token: &'a str,
    proxy: Option<&'a str>,
    master_chat_id: ChatId,
    prefix: Option<&'a str>,
}

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long, name = "PATH")]
    cfg_path: Option<String>,
    #[structopt(name = "MSG")]
    message: Option<String>,
    #[structopt(short, long, name = "FILE")]
    include: Option<String>,
}

// Just a wrapper for returning Strings as errors from main
struct Fin(String);

impl std::fmt::Debug for Fin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn main() -> Result<(), Fin> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let Args {
        cfg_path,
        message,
        include,
    } = Args::from_args();

    let cfg_path = cfg_path.map_or(
        config_dir().map_or(
            Err(Fin(
                "Can't obtain config directory. Specify path to config file with '-c'".to_owned(),
            )),
            |mut dir| {
                dir.push("notify-tg.toml");
                Ok(dir)
            },
        ),
        |s| Ok(std::path::PathBuf::from(s)),
    )?;

    let mut config_file = File::open(&cfg_path)
        .map_err(|err| Fin(format!("Can't open config ({:?}): {:?}", cfg_path, err)))?;

    let mut config_raw = String::new();
    config_file
        .read_to_string(&mut config_raw)
        .map_err(|err| Fin(format!("Can't read config: {:?}", err)))?;

    let Config {
        token,
        proxy,
        master_chat_id,
        prefix,
    } = toml::from_str(&config_raw)
        .map_err(|err| Fin(format!("Error parsing config: {:?}", err)))?;

    let bot = BotBuilder::new()
        .token(token)
        .client(match proxy {
            Some(proxy) => reqwest::Client::builder()
                .proxy(
                    reqwest::Proxy::https(proxy)
                        .map_err(|err| Fin(format!("Error creating reqwest::Proxy: {:?}", err)))?,
                )
                .build()
                .map_err(|err| Fin(format!("Error creating reqwest::Client: {:?}", err)))?,
            None => reqwest::Client::new(),
        })
        .build();

    let mut rt = tokio::runtime::Runtime::new().expect("create runtime");
    rt.block_on(async move {
        let act = match (message, include) {
            (Some(message), Some(include)) => {
                let message = prefix
                    .map_or_else(|| String::with_capacity(message.len()), str::to_owned)
                    + &message;
                bot.send_document(master_chat_id, InputFile::File(include.into()))
                    .caption(message)
                    .parse_mode(ParseMode::HTML)
                    .send()
                    .await
            }
            (Some(message), None) => {
                let message = prefix
                    .map_or_else(|| String::with_capacity(message.len()), str::to_owned)
                    + &message;
                bot.send_message(master_chat_id, message)
                    .parse_mode(ParseMode::HTML)
                    .send()
                    .await
            }
            (None, Some(include)) => {
                let res = bot.send_document(master_chat_id, InputFile::File(include.into()));
                match prefix {
                    Some(prefix) => res.caption(prefix),
                    None => res,
                }
                .parse_mode(ParseMode::HTML)
                .send()
                .await
            }
            (None, None) => {
                return match bot.get_me().send().await {
                    Ok(me) => {
                        log::info!("getMe -> {:#?}", me);
                        log::info!("Config is fine. Exiting.");
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        Err(Fin(e.to_string()))
                    }
                };
            }
        };
        if let Err(err) = act {
            log::error!("{}", err);
        }
        Ok(())
    })
}
