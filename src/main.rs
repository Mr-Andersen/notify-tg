use std::{fs::File, io::Read};

use dirs::config_dir;
use serde::Deserialize;
use structopt::StructOpt;
use teloxide::{
    prelude::*,
    types::{InputFile, ParseMode},
};

#[derive(Deserialize)]
struct Config<'a> {
    token: &'a str,
    proxy: Option<&'a str>,
    master_chat_id: teloxide::types::ChatId,
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
        .map_err(|e| Fin(format!("Can't open config ({:?}): {:?}", cfg_path, e)))?;

    let mut config_raw = String::new();
    config_file
        .read_to_string(&mut config_raw)
        .map_err(|e| Fin(format!("Can't read config: {:?}", e)))?;

    let Config {
        token,
        proxy,
        master_chat_id,
        prefix,
    } = toml::from_str(&config_raw).map_err(|e| Fin(format!("Error parsing config: {:?}", e)))?;

    let bot = Bot::with_client(
        token,
        match proxy {
            Some(proxy) => reqwest::Client::builder()
                .proxy(
                    reqwest::Proxy::https(proxy)
                        .map_err(|e| Fin(format!("Error creating reqwest::Proxy: {:?}", e)))?,
                )
                .build()
                .map_err(|e| Fin(format!("Error creating reqwest::Client: {:?}", e)))?,
            None => reqwest::Client::new(),
        },
    );

    let mut rt = tokio::runtime::Runtime::new().expect("Create runtime");
    let message = match message {
        Some(val) => prefix.map_or_else(|| String::with_capacity(val.len()), str::to_owned) + &val,
        None => {
            return rt.block_on(async move {
                match bot.get_me().send().await {
                    Ok(me) => {
                        log::info!("getMe -> {:#?}", me);
                        if let Some(_) = include {
                            log::warn!("`-i` flag received, but no message provided");
                        }
                        log::info!("Config is fine. Exiting.");
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        return Err(Fin(e.to_string()));
                    }
                }
            });
        }
    };

    rt.block_on(async move {
        match include {
            None => {
                bot.send_message(master_chat_id, message)
                    .parse_mode(ParseMode::HTML)
                    .send()
                    .await
                    .log_on_error()
                    .await
            }
            Some(filename) => {
                bot.send_document(master_chat_id, InputFile::File(filename.into()))
                    .caption(message)
                    .parse_mode(ParseMode::HTML)
                    .send()
                    .await
                    .log_on_error()
                    .await
            }
        }
    });

    Ok(())
}
