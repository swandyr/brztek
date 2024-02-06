# brztek

## Setup

Install `sqlx-cli` and use this commands to create and initialize the database.

    $ sqlx create database
    $ sqlx migrate run

Create a `.env` file.

    DISCORD_TOKEN=<TOKEN>
    DATABASE_URL="sqlite:database.sqlite"
    RUST_LOG=warn

config.toml
```toml
database = "sqlite:database.sqlite"

[brzthook]
port = # The port used by the listener
ip_addr = # The address to bind the TCP listener
callback = # The address passed to the hub
new_only = # true/false; notify only new videos
```

### Help
    Commands:
      /help                   
    
    Admin:
      /import_mee6_levels     Import users levels from Mee6 leaderboard
      /set_xp                 Set the user's xp points
    
    Levels:
      /rank                   Show your rank
      /top                    Show the top users of the server
    
    Mention Roles:
      /gimmeroles             Get roles to be mentionned
      /mention_roles          Manage mention roles (require MANAGE_ROLES permission)
      /mention_roles create   Create a new role as a mention role managed by the bot
      /mention_roles delete   Delete a mention role from the bot and discord
      /mention_roles add      Add an existing role to the mention roles managed by the bot
    
    Misc:
      /br                     Check if Jolene is playing on BigRig FM
      /clean                  Explicitly call clean_url
      /learn                  Make the bot remember.
      /learned                What the bot learned.
      /ping                   Ping the bot!
      /setcolor               Get your own role
    
    Roulette:
      /rffstar                Who goes the highest before trigerring RFF ?
      /roulette               Put random member in timeout for 60s
      /statroulette           Shows some statistics about the use of roulettes
      /toproulette            Roulette Leaderboard
    
    Youtube:
      /yt                     Commands for interacting with Youtube
      /yt search              Search a Youtube video.
      /yt sub                 Create a new Youtube webhook
      /yt unsub               Unsub and delete a webhook
      /yt list                List all subs in the guild
      /yt sub_details         

### ClearUrl

Use rules from the [ClearUrls addon](https://github.com/ClearURLs/Addon):
[rules](https://rules2.clearurls.xyz/data.minify.json)

Python implementation: (https://gitlab.com/CrunchBangDev/cbd-cogs/-/blob/master/Scrub/scrub.py) I mostly rewritten.
