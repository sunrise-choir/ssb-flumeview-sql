# ssb-flume-follower-sql 

> (WIP) Node bindings to a sql view of a ssb database 

- sqlite3 based db
- the flume offset log is the source of truth. This reads the offset log and builds the db from it.
  - does not modify the offset log.
- clients using this module are free to create their own sqlite file in their application folder, _outside_ of the .ssb folder. Clients don't need to agree on flume index versions to run multiple clients at the same time. 
- supports building the db in chunks to control cpu use. 
- supports querying the db at any time. The db does not have to be up to date with the offset log. 
- decrypts private messages.  
- uses knex for powerful query building.
- lots of knex helpers / sql views to use as building blocks for queries.
- FAST. Fast to build the db. Fast to query. Benchmarked.
- Friendly. We have a code of conduct and will enforce it.
  - Found something missing from the docs? Can't understand the code? Found an un-tested edge case? Spotted a poorly named variable? Raise an issue! We'd love to help.
- well tested
- well documented

## Example

```js
var pull = require('pull-stream')
var Flume = require('flumedb')
var db = Flume(...) // configure flume
var SqlView = require('ssb-flumeview-sql')

var sqlView = SqlView('/path/to/log.offset', '/path/to/view.sqlite') 

// So far, nothing much has happened. A new sqlite db is created if it doesn't exist. No indexing is happening automatically.

console.log(sqlView.getLatest()) => 100 // Last time this ran, it inserted up to sequence number 100 in the view.
console.log(db.since.value) => 1000 // Ok, the flume db has up to sequence 1000, so the view is behind the log.

// Create a query that will get us whatever is in the view right now.
sqlView.query({ query: "SELECT * message WHERE content_type='post'" }, function(err, result){
  // => Immediately logs results of the query
  console.log(result)
})

// Create a query that will wait until the view is up to a certain sequence 
sqlView.query({ query: "SELECT * message WHERE content_type='post'", whenUpToSequence: 1000 }, function(err, result){
  console.log(result)
})

// Let's get some more data into the view, but throttled so it's not too cpu hungry. (Assumes you can use `requestIdleCallback`)
requestIdleCallback(function(deadline){
  while(deadline.timeRemaing > 0 || deadline.didTimeout){
    sqlView.process({chunkSize: 500})
  }
})

// The query waiting for sequence number 1000 will eventually call back when enough items are added to the view.

```

## Performance

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
