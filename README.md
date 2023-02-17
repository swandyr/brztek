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

### Chat commands (prefix `$`)
- `$am_i_admin` : check for administrator permissions
- `$delete_ranks`  clear all entries in the config table for the guild
- `$config <param> <value>` : get the value of a parameter, set if `value` is provided
    - `spam_delay`
    - `min_xp_gain`
    - `max_xp_gain`
- `$ping` : pong!
- `$learn <name> <link>` : save a link that can be called with `$*name*`
- `$learned` : list of all commands saved using `$learn`
- `$rank` : show user's rank card
- `$top <x>` : show a card of `x` most active users
- `$help`


### Slash commands
- `/set <xp> <messages> <user>` : set the values of xp and messages counts to a user
- `/pub <channel>` : Set the public channel where are send the welcome messages
- `/learn <name> <link>` : as above

<!-- TODO: Add possibility to add fonts, profile pictures...>
<!-- TODO: Add logging to file>
<!-- TODO: Possibility to set xp to users OR find a way to get meee6 leaderboard>
<!-- TODO: reaction roles>
<!-- TODO: Try Piet (https://github.com/linebender/piet) to replace raqote (issue with text drawing)>
<!-- TODO: Migrate to slash commands>
