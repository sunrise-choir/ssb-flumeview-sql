# ssb-flume-follower-sql 

> work in progress

A sql-based database for secure scuttlebutt, written in rust, with bindings to js

This is conceptually very similar to a [flume-view](https://github.com/flumedb/flumedb#views) but **it isn't a flume view that plugs into the rest of flume**.
This module parses the [flume append only log file](https://github.com/flumedb/flumelog-offset) and inserts each message into a sql database.

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
const SqlView = require('ssb-flumeview-sql')
const config = // load ssb config 
const keys =  // load ssb keys
const logPath = Path.join(config.path, 'flume', 'log.offset')
const secret = ssbKeys.ssbSecretKeyToPrivateBoxSecret(keys)

// Constructing a new sqlView doesn't do that much automatically. A new sqlite db is created if it doesn't exist. No indexing is happening automatically.
const sqlView = SqlView(logPath, '/tmp/patchwork.sqlite3', secret, keys.id)

// The sql view has the knex instance and some useful strings and knex modifiers attached.
var { 
  knex, 
  modifiers, 
  strings 
  } = sqlView
var { links } = strings //links is a string constant of the links table name.
var { backLinksReferences } = modifiers
var id = "%E1d7Dxu+fmyXB7zjOMfUbdLU8GuGLRQXdrCa0+oIajk=.sha256"

//Query for backlinks (same as backlinks references query used in patchwork)
knex
  .select([
    'links.link_from as id',
    'author',
    'received_time as timestamp'
  ])
  .from(links)
  .modify(backLinksReferences, id, knex)
  .asCallback(function(err, result) {
    console.log(result) // => [{id: "...", author: "...", timestamp: "..."}]
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

## API

```js
var SqlView = require('ssb-flumeview-sql')
var sqlView = SqlView('/path/to/log.offset', '/path/to/view.sqlite', <pubKey>, <secreKey> ) 
```

### sqlView.process(opts = {})

`opts` is mandatory and has one optional field:

- `opts.chunkSize` (optional) - Sets the maximum number of items to process. If this is omitted it will process all entries, bringing the view up to date with the log.

- Note that processing will block this thread while executing. If you want to limit resource use of processing, use something like `requestIdleCallback` like in the example. Also be careful not to make `opts.chunkSize` too large. As a starting point, my machine processes 10000 entries in 140ms.

### sqlView.getLatest()

Gets the latest flume sequence value processed by the db.

### sqlView.knex

A knex instance ready to do **read only** queries on the db. TBD if I can get knex writes working. Sqlite theortically supports multiple db connections.

### sqlView.modifiers

TBD if these are a good idea. They are syntactic sugar for common queries using [knex modify](https://knexjs.org/#Builder-modify). I'll know more once I write a whole lot of queries for patchwork.

## More Example Queries

### My most recent 20 posts

### My most recent posts, pages of 20.

### All the authors that liked this message

### Authors I block

### All the about messages about me

### All recent posts by me and people I follow (1 hop)

### How many mentions do I have since a given flume sequence

### What messages reference a given blob

### Which authors reference a given blob

### Which blobs haven't been referenced by me or people I follow in the last year.

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
$ npm install 
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

@mixmix for awesome feedback on the readme.

## See Also

## [Code of Conduct](/code-of-conduct.md)

## License

AGPL3
