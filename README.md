![Teloxide](https://img.shields.io/badge/Powered%20by-Teloxide-red)
# Installation & usage
  - Create file `notify-tg.toml` in your "config" directory (on Linux: `$HOME/.config/`), copy-paste contents of `example-config.toml` there
  - Create bot with [@BotFather](https://t.me/BotFather). Put token in the config
  - Write something to your bot (`/start`), so it can answer back later
  - Find out your `chat_id` (you can do that with [@showjsonbot](https://t.me/showjsonbot) for example)
  - Put this `id` into config
  - You can set `prefix` -- this line will be prepended to all your messages
  - Optionally, set/remove proxy
  - Do `cargo install --git https://github.com/Mr-Andersen/notify-tg && notify-tg "notify-tg has been successfully installed!"`
  - When installation finishes, bot will send a message to you. Now when you do `notify-tg "Message"`, bot sends to you "Message"
# Lib
https://github.com/teloxide/teloxide
# Comment
Yet, compilation time is quite high. I hope to reduce it at some point, by using only part of `teloxide` (like, its core) and probably something lighter than `clap` for CLI
