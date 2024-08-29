# Rusteam

Pull steam game playtime information, wishlists, and notes from Steam and Notion and combine
them in interesting ways.

A hobby project to make it even easier to spend almost as much time figuring out which game
to play as actually playing a game.

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
