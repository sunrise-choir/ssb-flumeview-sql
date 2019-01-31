# ssb-flume-follower-sql 

> (WIP) A sql view of a ssb database that follows the offset log. Written in rust with bindings to js. 

- [Sqlite3](http://sqlite.org) based db
- The [flume offset log](https://github.com/flumedb/flumelog-offset) is the source of truth. This module reads the offset log and builds the db from it.
  - Does not modify the offset log.
- Clients using this module are free to create their own sqlite file in their application folder, _outside_ of the .ssb folder. Clients don't need to agree on db versions to be able to run multiple clients at the same time. 
- Uses [knex](http://knexjs.org/) for powerful async queries.
- Supports querying the db at any time. The db does not have to be up to date with the offset log. 
- Supports processing the offset log in chunks to control cpu use. 
- The [Schema](#Schema) models all the common relationships between messages. This enables some powerful queries that have been hard to do in the existing stack. 
- Decrypts private messages using [private-box](https://github.com/pietgeursen/private-box-rs) in rust.  
- Lots of knex helpers / sql views to use as building blocks for queries.
- [FAST](#Performance). Fast to build the db. Fast to query. Benchmarked.
- Friendly. We have a [code of conduct](/code-of-conduct.md) and are commited to abiding by it.
  - Found something missing from the docs? Can't understand the code? Found an un-tested edge case? Spotted a poorly named variable? Raise an issue! We'd love to help.
- Well tested.
- Well documented.

## Example

```js
const config = // load ssb config 
const keys =  // load ssb keys
const logPath = Path.join(config.path, 'flume', 'log.offset')
const secret = ssbSecretKeyToPrivateBoxSecret(keys)

const sqlView = SqlView(logPath, '/tmp/patchwork.sqlite3', secret, keys.id)
// So far, nothing much has happened. A new sqlite db is created if it doesn't exist. No indexing is happening automatically.

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

//Query for backlinks (same as backlinks references query used in patchwork)

var { knex, modifiers, strings } = sqlView
var { links } = strings
var { backLinksReferences } = modifiers
var id = "%E1d7Dxu+fmyXB7zjOMfUbdLU8GuGLRQXdrCa0+oIajk=.sha256"

knex
  .select([
    'links.link_from as id',
    'author',
    'received_time as timestamp'
  ])
  .from(links)
  .modify(backLinksReferences, id, knex)
  .asCallback(function(err, result) {
    console.log(result)
  })
```

## Performance

### Building the db

### Querying

### Disk use

## Schema

![schema](/docs/images/ssb-flumeview-sql.jpg)

## API

```js
var SqlView = require('ssb-flumeview-sql')
var sqlView = SqlView('/path/to/log.offset', '/path/to/view.sqlite') 
```

### sqlView.query(opts = {}, cb)

`opts` is mandatory and has some required and optional fields:

- `opts.query` (required) - sql query string.

- `opts.whenUpToSequence` (optional) - sequence number the view must be up to before running the query. Omitting this means the query will be executed immediately, even though the view might be behind the log.

`cb` is a node-style, error first callback:

```js
function cb (err, results){
  if(err){
    //handle the error
  }
  
  //results is an array
  results.forEach(console.log)
}
```

### sqlView.process(opts = {})

`opts` is mandatory and has one optional field:

- `opts.chunkSize` (optional) - Sets the maximum number of items to process. If this is omitted it will process all entries, bringing the view up to date with the log.

- Note that processing will block this thread while executing. If you want to limit resource use of processing, use something like `requestIdleCallback` like in the example. Also be careful not to make `opts.chunkSize` too large. As a starting point, my machine processes 10000 entries in 140ms.

## Install

With [npm](https://npmjs.org/) installed, run

```
$ npm install 
```

## Building

### Build for your environment

Dev Dependencies:
  - rust + cargo
  - cargo-make
  - cmake

```
$ npm run prebuild
```

### Todo

- [x] remove since from the api
- [ ] documentation needs to have the table structure so you can know how to query. Think this means that that rust needs to live in that project.
- [ ] write convenience methods that wrap common queries.
- [ ] propogate file open errors up to the js
- [x] refactor structure of flume db. Pull out ssb specific stuff.
- [x] refactor prebuild stuff


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

## See Also

## License

AGPL3
