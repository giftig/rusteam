# Rusteam

Pull steam game playtime information, wishlists, and notes from Steam and Notion and combine
them in interesting ways.

A hobby project to make it even easier to spend almost as much time figuring out which game
to play as actually playing a game.

## Scraping

### Commands

For basic usage, see `rusteam --help`.

#### Sync

The basic command is `sync`; this will perform all the primary sync tasks, scraping steam API
data and notion wishlist data to populate the relevant tables in postgres, as well as
syncing some data back to the notion wishlist.

Data synced back into notion includes:
  - updating release status when a game is newly released
  - adding app IDs by performing a name match against the `steam_game` table where possible.
    An exact name match is required, there are too many games on steam to do a fuzzy match.

The scraper will also print some useful info as it processes data. The most useful of these
notifications are:
  - A game has been released: i.e. Notion had it listed as unreleased but steam API shows it's
    since been released
  - The "release date" text in Steam has changed; this can either mean the release date is
    being narrowed down, e.g. `"Coming soon" -> "18 Sep, 2025"` or that it's been postponed,
    e.g. `"17 Jan 2025" -> "2026"`. Note that a best-effort approach is made to turn this
    string into an actual timestamp to allow sorting by the field. See
    `steam::conv::parse_release_date`.

#### Import wishlist

Wishlist APIs are not made public by Steam, so for private wishlists the only way of scraping
them would be to provide a cookie from a valid login session. Retrieving such a cookie is likely
to be unreliable given Steam uses MFA and doesn't encourage scraping its web APIs.

Currently the `import-wishlist` command just supports syncing wishlist data to the `wishlist`
table for a provided `wishlist.json` file, and you'll need to fetch that file yourself.

The easiest way to do this is:
  1. Log in to steam
  2. Go to your wishlist page: `https://store.steampowered.com/wishlist/id/<username>/`
  3. In the network tab, look for requests retrieving JSON files; there should only be 2 for
     this page, one for wishlist and one for general userdata
  4. Retrieve the payload for the request to `/wishlist/profiles/<id>/wishlistdata/`

You can then import the wishlist with `rusteam import-wishlist -f wishlist.json`.

## Queries

Some useful queries for analysing upcoming games, recently-played games, and other interesting
data collected are included as `*.sql` files under `queries`.

### Analysis with superset

I've started using Apache Superset to build some charts, tables, and dashboards using these
queries, and geared the docker-compose which runs the postgres db towards sharing its network
with superset running in its own docker-compose.

To make starting superset easier, the `superset` subdirectory contains a fork of the official
`docker-compose-image-tag.yml` and its dependencies from the `apache/superset` repo, with
some modifications to allow easily connecting to the rusteam network. You can easily control
this with `make superset-up` and `make superset-down` or by descending into the `superset`
directory and working with `docker compose` directly.

On first start you'll need to connect the `rusteam` postgres database as a data source,
using:

```
host = rusteam_db
port = 5432
database = rusteam
username = admin
password = admin
```

You can then start writing your own queries and charts, or copy examples from `queries/`
into SQL Lab to experiment with them.
