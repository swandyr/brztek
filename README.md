# brztek

## Before start

Install `sqlx-cli` and use this commands to create and initialize the database.

    $ sqlx create database
    $ sqlx migrate run

Create a `.env` file.

    DISCORD_TOKEN=<TOKEN>
    DATABASE_URL="sqlite:database.sqlite"
    RUST_LOG=warn


## Commands

### General
- `ping` : pong!
- `learn <name> <link>` : save a link that can be called with `$*name*`
- `learned` : list of all commands saved using `$learn`
- `bigrig` : show the current that is playing on BigRig FM

### Levels
- `rank` : show user's rank card
- `top <x>` : show a card of `x` most active users

### Admin
- `delete_ranks`  clear all entries in the config table for the guild
- `admin <subcommand>` : Change server configuration
    - `spam_delay <int>` : Consecutives messages sent below this delay will not grant xp points
    - `min_xp_gain <int>` : Minimal xp points gained per message
    - `max_xp_gain <int>` : Maximal xp points gained per message
    - `set_pub <channel>` : Set a channel where the bot will sent welcome messages
    - `set_user <user> <int>` : Set the amount of xp points of a user
- `import_mee6_levels` : Automatically import levels from Mee6 (only with slash command)

### Help
- `help`


<!-- TODO: Add logging to file>
<!-- TODO: Add create  color roles command>
<!-- TODO: Reaction roles>
<!-- TODO: Round corners of avatar in rank card>
<!-- TODO: Maybe find something to replace raqote>
