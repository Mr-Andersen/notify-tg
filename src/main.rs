use std::{fs::File, io::Read};

use clap::{App, Arg};
use dirs::config_dir;
use serde::Deserialize;
use teloxide::prelude::*;

#[derive(Deserialize)]
struct Config<'a> {
    token: &'a str,
    proxy: Option<&'a str>,
    master_chat_id: teloxide::types::ChatId,
}

// Just a wrapper for returning Strings as errors from main
struct Fin(String);

impl std::fmt::Debug for Fin {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn main() -> Result<(), Fin> {
    let args = App::new("notify-tg")
        .version("1.0.0")
        .arg(
            Arg::with_name("cfg_path")
                .short("c")
                .long("cfg_path")
                .value_name("PATH")
                .help(&format!(
                    "provide path to config file (defaults to {})",
                    config_dir().map_or("(unavailable)".to_owned(), |pb| pb
                        .to_string_lossy()
                        .to_string()
                        + "/notify-tg.toml")
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("message")
                .help("message to send; if omitted, validate config and exit")
                .index(1)
                .takes_value(true),
        )
        .get_matches();

    let config_path = args.value_of("cfg_path").map_or(
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
    let message = args.value_of("message");

    let mut config_file = File::open(&config_path).map_err(|e| {
        Fin(format!("Can't open config ({:?}): {:?}", config_path, e))
    })?;

    let mut config_raw = String::new();
    config_file
        .read_to_string(&mut config_raw)
        .map_err(|e| Fin(format!("Can't read config: {:?}", e)))?;

    let Config {
        token,
        proxy,
        master_chat_id,
    } = toml::from_str(&config_raw)
        .map_err(|e| Fin(format!("Error parsing config: {:?}", e)))?;

    let bot = Bot::with_client(
        token,
        match proxy {
            Some(proxy) => reqwest::Client::builder()
                .proxy(
                    reqwest::Proxy::https(proxy)
                        .map_err(|e| Fin(
                            format!("Error creating reqwest::Proxy: {:?}", e)
                        ))?
                )
                .build()
                .map_err(|e| Fin(
                    format!("Error creating reqwest::Client: {:?}", e)
                ))?,
            None => reqwest::Client::new(),
        },
    );

    let message = match message {
        Some(val) => val,
        None => {
            eprintln!("Config is fine. Exiting.");
            return Ok(());
        }
    };

    teloxide::enable_logging!();
    tokio::runtime::Runtime::new()
        .expect("Create runtime")
        .block_on(async move {
            bot.send_message(master_chat_id, message)
                .send()
                .await
                .log_on_error()
                .await
        });

    Ok(())
}
