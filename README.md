# brztek

## Setup

Install `sqlx-cli` and use this commands to create and initialize the database.

    $ sqlx create database
    $ sqlx migrate run

Create a `.env` file.

    DISCORD_TOKEN=<TOKEN>
    DATABASE_URL="sqlite:database.sqlite"
    RUST_LOG=warn


### Help
- `/help`

### ClearUrl

Use rules from the [ClearUrls addon](https://github.com/ClearURLs/Addon):
[rules](https://rules2.clearurls.xyz/data.minify.json)

Python implementation: (https://gitlab.com/CrunchBangDev/cbd-cogs/-/blob/master/Scrub/scrub.py) I mostly rewritten.