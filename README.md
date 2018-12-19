# ssb-flumeview-sql 

> Node bindings to to sql flumeview on a ssb database 

- differs from the current idea of a flumeview
  - doesn't plug into a js flume log, uses it's own internal rust based flumelog
  - does not modify the offset log.
  - supports building the view in chunks to control cpu use and allow queries to be handled in between building the view.

- supports querying the view, even if the view is behind the log.

## Example

```js
var pull = require('pull-stream')
var Flume = require('flumedb')
var db = Flume(...) // configure flume
var SqlView = require('ssb-flumeview-sql')

var sqlView = SqlView('/path/to/log.offset', '/path/to/view.sqlite') 

//So far, nothing much has happened. A new sqlite db is created if it doesn't exist. No indexing is happening automatically.

console.log(sqlView.getLatest()) => 100 // Let's say that last time this ran, it inserted up to sequence number 100 in the view.
console.log(db.since.value) => 1000 // Ok, the flume db has up to sequence 1000, so the view is behind.

//Let's create a query that will get us whatever is in the view right now.
sqlView.query({ query: "SELECT * message WHERE content_type='post'" }, function(err, result){
  // => Immediately logs results of the query
  console.log(result)
})

//Let's create a query that will wait until the view is up to a certain sequence 
sqlView.query({ query: "SELECT * message WHERE content_type='post'", whenUpTo: 1000 }, function(err, result){
  console.log(result)
})

// Let's get some more data into the view, but throttled so it's not too cpu hungry. (Assumes you can use `requestIdleCallback`)
requestIdleCallback(function(deadline){
  while(deadline.timeRemaing > 0 || deadline.didTimeout){
    sqlView.process({chunkSize: 100})
  }
})

// The query waiting for sequence number 1000 will eventually callback when enough items are added to the view.

```

## API

```js
var SqlView = require('ssb-flumeview-sql')
var sqlView = SqlView('/path/to/log.offset', '/path/to/view.sqlite') 
```

### sqlView.query(opts = {}, cb)

`opts` is mandatory and has some required and optional fields:

- `opts.query` (required) - sql query string.

- `opts.whenUpTo` (optional) - sequence number the view must be up to before running the query. Omitting this means the query will be executed immediately, even though the view might be behind the log.

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

### sqlView.process(opts = {}, cb)

`opts` is mandatory and has some required and optional fields:

- `opts.chunkSize` (optional) - Sets the maximum number of items to process. If this is omitted it will process all entries, bringing the view up to date with the log.

`cb` is optional.

- If cb is provided the query will execute asynchronously on a libuv thread. Useful if you want to utilise multiple processors.
- If cb is not provided then processing will block this thread while executing. This is useful if you want to limit resource use of processing using something like `requestIdleCallback` like in the example.

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
