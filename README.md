![Teloxide](https://img.shields.io/badge/Powered%20by-Teloxide-red)

# Why?
I often run heavy computations that take some time -- usually couple of hours. I want to know when computation finishes as soon as possible. And I also use Telegram on daily basis. That's why I've created `notify-tg`:
  - `./long-running-something && notify-tg 'Calculation successfull' || notify-tg 'Calculation failed'` -- receive notification about calculation status
  - from python -- same as previous, but include constants in message:
  ```Python3
  subprocess.run(['notify-tg', 'calc finished for <pre>' + constants_prettified + '</pre>'])
  ```

# Installation
  - Create file `notify-tg.toml` in your "config" directory (on Linux: `$HOME/.config/`), copy-paste contents of `example-config.toml` there
  - Create bot with [@BotFather](https://t.me/BotFather). Put token in the config
  - Write something to your bot (`/start`), so it can answer back later
  - Find out your `chat_id` (you can do that with [@showjsonbot](https://t.me/showjsonbot))
  - Put this `id` into config
  - You can set `prefix` -- this line will be prepended to all your messages
  - Optionally, set/remove proxy
  - Do `cargo install --git https://github.com/Mr-Andersen/notify-tg && notify-tg "notify-tg has been successfully installed!"`
  - When installation finishes, bot will send a message to you. Now when you can send message to yourself with `notify-tg "Message"`

# Features
  - Validate your config and connection to Telegram servers with running `notify-tg` without arguments. On success it will show you `getMe` response -- basic info about your bot, such as its `@username`.
  - `-i|--include [FILE]` flag for sending file captioned with `MSG`
  - Messages are html

# Lib
https://github.com/teloxide/teloxide

# Comment
Yet, compilation time is quite high. I hope to reduce it at some point, by using only part of `teloxide` (like, its core) and probably something lighter than `structopt` for CLI
