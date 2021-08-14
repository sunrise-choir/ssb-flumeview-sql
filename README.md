# ssb-flume-follower-sql 

> (work in progress :construction: ) A sql-based database for secure scuttlebutt, written in rust, with bindings to js

This module parses the [flume append only log file](https://github.com/flumedb/flumelog-offset) and inserts each message into a sql database.

This is conceptually very similar to a [flume-view](https://github.com/flumedb/flumedb#views) but **it isn't a flume view that plugs into the rest of flume**. [@mmckegg](https://github.com/mmckegg) wrote a [post](https://viewer.scuttlebot.io/%251bz0TXDuaM65KMTb8bgtrXuqD7L77baneTdoJ0EwRug%3D.sha256) about his dream of a "consensus free scuttlestack." This module intenionally isn't a flume view because we want clients to be able to take ownership of the indexes they need without needing consensus about how they modify the .ssb folder.

## Contents

- [Features](#features)
- [Example](#example)
- [Schema](#schema)
- [API](#api)
- [More Example Queries](#more-example-queries)
- [Hints For Developing Queries](#hints-for-developing-queries)
- [Performance](#performance)
- [Development](#development)
- [Acknowledgments](#acknowledgments)
- [See Also](#see-also)
- [Code of Conduct](#code-of-conduct)

## Features

- Easy to use
  - [models all the common relationships](#Schema) between messages, enabling powerful queries that are generally hard to do
  - uses [knex](http://knexjs.org/) for powerful async queries
- Fast + flexible
  - built on Rust + [Sqlite3](http://sqlite.org)
  - supports processing the log and inserting into the db in spare cpu time, so your application never chokes
  - bulding + queries are [FAST](#Performance) (benchmarked!)
- Advanced features
  - query the view at any time - for when waiting for the view to be in sync isn't relevant
  - supports multiple identities (WIP) (multiple keypairs), with [decryption done with Rust](https://github.com/pietgeursen/private-box-rs)
  - supports multiple clients running different versions of this view, by giving you control over where this view is saved
- Friendly. We have a [code of conduct](/code-of-conduct.md) and are commited to abiding by it.
  - Found something missing from the docs? Can't understand the code? Found an un-tested edge case? Spotted a poorly named variable? Raise an issue! We'd love to help.
- Well tested
- Well documented

## Example

```js
const SqlView = require('ssb-flume-follower-sql')
const config = // load ssb config 
const keys =  // load ssb keys
const logPath = Path.join(config.path, 'flume', 'log.offset')
const secret = ssbKeys.ssbSecretKeyToPrivateBoxSecret(keys)

// Constructing a new sqlView doesn't do that much automatically. A new sqlite db is created if it doesn't exist. No indexing is happening automatically.
const sqlView = SqlView(logPath, '/tmp/patchwork.sqlite3', secret, keys.id)

// The sql view has the knex instance 
var { knex } = sqlView

//Query for content of 20 most recent posts. 
knex
  .select(['content'])
  .from('messages_raw')
  .join('authors', 'messages_raw.author_id', 'authors.id')
  .orderBy('flume_seq', 'desc')
  .limit(20)
  .asCallback(function(err, result) {
    console.log(result) 
  })

// Process data from the offset log into the db as it comes in, but throttled so it's not too cpu hungry. (Assumes you can use `requestIdleCallback`)
window.requestIdleCallback(function processMore (deadline) {
  window.requestIdleCallback(processMore)

  //uses the latest sequence from flumedb to check if there are new things in the offset log.
  const sbotLatest = api.sbot.obs.latestSequence()()

  if (sqlView.getLatest() === sbotLatest) return

  // Process chunks until we've used up the available free time.
  while (deadline.timeRemaining() > 0) {
    sqlView.process({ chunkSize: 250 })
  }

  sqlViewLatest = sqlView.getLatest()
})
```

See more [example queries below](#more-example-queries)

## Schema

### Tables:

![schema](/docs/images/ssb-flumeview-sql.jpg)

### Views

[sql views](https://en.wikipedia.org/wiki/View_(SQL)) of the db that do joins you're likely to use.

#### messages

![messages-view](/docs/images/messages-view.jpg)

Query that generates view:

```sql
SELECT 
  flume_seq,
  key_id,
  seq,
  received_time,
  asserted_time,
  root_id,
  fork_id,
  author_id,
  content,
  content_type,
  is_decrypted,
  keys.key as key,
  root_keys.key as root,
  fork_keys.key as fork,
  authors.author as author
FROM messages_raw 
JOIN keys ON keys.id=messages_raw.key_id
LEFT JOIN keys AS root_keys ON root_keys.id=messages_raw.root_id
LEFT JOIN keys AS fork_keys ON fork_keys.id=messages_raw.fork_id
JOIN authors ON authors.id=messages_raw.author_id
```

#### `links`

#### `abouts`

#### `contacts`

#### `mentions`




## API

```js
var SqlView = require('ssb-flume-follower-sql')
var sqlView = SqlView('/path/to/log.offset', '/path/to/view.sqlite', <pubKey>, <secreKey> ) 
```


### sqlView.process(opts = {})

`opts` is mandatory and has one optional field:

- `opts.chunkSize` (optional) - Sets the maximum number of items to process. If this is omitted it will process all entries, bringing the view up to date with the log.

- Note that processing will block this thread while executing. If you want to limit resource use of processing, use something like `requestIdleCallback` like in the example. Also be careful not to make `opts.chunkSize` too large. As a starting point, my machine processes 10000 entries in 140ms.

### sqlView.getLatest()

Gets the latest flume sequence value processed by the db.

### sqlView.knex

A knex instance ready to do **read only** queries on the db. 

- TBD if I can get knex writes working. Sqlite theortically supports multiple db connections.
- TBD if you _should_ mess with the db. 

### sqlView.modifiers

TBD if these are a good idea. They are syntactic sugar for common queries using [knex modify](https://knexjs.org/#Builder-modify). I'll know more once I write a whole lot of queries for patchwork.

## More Example Queries

### Content of my most recent 20 posts

```js
db.knex
  .select(['content', 'author'])
  .from('messages_raw')
  .join('authors', 'messages_raw.author_id', 'authors.id')
  .where('authors.is_me', 1)
  .orderBy('flume_seq', 'desc')
  .limit(20)
```

### Content of my most recent posts, pages of 20, third page.

```js
db.knex
  .select(['content', 'author'])
  .from('messages_raw')
  .join('authors', 'messages_raw.author_id', 'authors.id')
  .where('authors.is_me', 1)
  .orderBy('flume_seq', 'desc')
  .limit(20)
  .offset(40)
```

### Authors I block

```js
db.knex
  .select(['author'])
  .from('contacts_raw')
  .join('authors', 'contacts_raw.author_id', 'authors.id')
  .where('authors.is_me', 1)
  .where('state', -1)
```

### Authors I follow

```js
db.knex
  .select(['author'])
  .from('contacts_raw')
  .join('authors', 'contacts_raw.author_id', 'authors.id')
  .where('authors.is_me', 1)
  .where('state', 1)
```

### How many mentions do I have since a given flume sequence

```js
var last_seq_num = 12345

db.knex('mentions_raw')
  .count('id')
  .join('authors', 'mentions_raw.link_to_author_id', 'authors.id')
  .join('messages_raw', 'mentions_raw.link_from_key_id', 'messages_raw.key_id')
  .where('authors.is_me', 1)
  .where('messages_raw.flume_seq', '>', last_seq_num )
```

### All the about messages about me

```js
db.knex
  .select(['content'])
  .from('abouts_raw')
  .join('authors', 'abouts_raw.link_to_author_id', 'authors.id')
  .join('messages_raw', 'abouts_raw.link_from_key_id', 'messages_raw.key_id')
  .where('authors.is_me', 1)
```

### All recent posts by me and people I follow (1 hop)

TODO

### What messages reference a given blob

TODO

### Which authors reference a given blob

TODO

### Which blobs haven't been referenced by me or people I follow in the last year.

TODO

## Hints for developing queries:

- install `sqlite3` on your system.
- in the sqlite command prompt run:
  - `sqlite> .timer on`. The timer lets you see how long your queries are taking. There are often multiple ways to write a query. Try them out to see which one is faster. If something is slow, pay attention to which columns are indexed (marked with `IDX` in the schema diagram.) 
  - `sqlite> .headers on`. Gives you column header names at the top of your results.

## Performance

### Building the db:

Roughly 10x faster!

Sqlite db rebuild as fast as possible: 40s 

Sqlite db rebuild, chunks of 250, **running on patchwork's main thread without lagging the ui**: 64s.

Flume rebuild of indexes used by patchwork: 404s


NB: This is a bit hard to do an exact comparison. Expect these numbers to change.

### Querying:

WIP.

From rust (not using knex) querying for all authors I block on a db of 100k messages:
54 micro seconds.

### Disk use:

Roughly 65% of the offset log. 

Roughly 50% of the existing flume indexes.

My Sqlite db is 379MB.

My offset log is 598MB, 

My flumedb indexes are 771MB.

## Install

With [npm](https://npmjs.org/) installed, run

```
$ npm install ssb-flume-follower-sql
```

## Development

### Build for your environment

Dev Dependencies:
  - rust + cargo
  - cargo-make
  - cmake

```
$ ./build-native.sh
```

### Cross compile

Dev Dependencies
  - same as above
  - cross
  - dockcross

Cross compiling is still a work in progress. Still todo:
- [ ] split out scripts to cross in build-native.sh
- [ ] add script to tar up cross compiled binding with correct prebuild compatible filename
- [ ] windows builds don't work

## Acknowledgments

- [@ahdinosaur](https://github.com/ahdinosaur) for orchestrating the sunrise choir. 
- [@mmckegg](https://github.com/mmckegg) for the infectious enthusiasm.
- [@mixmix](https://github.com/mixmix) for awesome feedback on the readme.
- [@dominictarr](https://github.com/dominictarr) for making it work before making it nice.
- [@AljoschaMeyer](https://github.com/AljoschaMeyer) for making it nice before making it work.
- [@noffle](https://github.com/noffle) for [common-readme](https://github.com/noffle/common-readme).

All the lovely people on scuttlebutt who make it the place it is.

## See Also

- [flumedb](https://github.com/flumedb/flumedb)
- [ssbc](https://github.com/ssbc)
- [sunrise choir](https://github.com/sunrise-choir)

## [Code of Conduct](/code-of-conduct.md)

## License

LGPL-3.0
