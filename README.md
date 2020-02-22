![Teloxide](https://img.shields.io/badge/Powered%20by-Teloxide-red)
# Installation & usage
  - Create file `notify-tg.toml` in your "config" directory (on Linux: `$HOME/.config/`), copy-paste contents of `example-config.toml` there
  - Create bot with [@BotFather](https://t.me/BotFather). Put token in the config
  - Write something to your bot (`/start`), so it can answer back later
  - Find out your chat_id (you can do that with @showjsonbot for example)
  - Do `cargo install --git https://github.com/Mr-Andersen/notify-tg && notify-tg "notify-tg has been successfully installed!"`
  - When installation finishes, bot will send a message to you. Now when you do `notify-tg "Message"`, bot sends to you "Message"
# Lib
https://github.com/teloxide/teloxide
