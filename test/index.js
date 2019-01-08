var test = require('tape')
var Db = require('../')
var rimraf = require('rimraf')

test('create', function (t) {
  var db = Db('/tmp/test.offset', '/tmp/test.sqlite')
  t.ok(db)
  t.end()
})

test('db has method getLatest ', function (t) {
  var db = Db('/tmp/test.offset', '/tmp/test.sqlite')
  t.equal(typeof (db.getLatest()), 'number')
  t.end()
})

test.skip('db has method query ', function (t) {
  var db = Db('/tmp/test.offset', '/tmp/test.sqlite')
  t.equal(typeof (db.query), 'function')
  t.end()
})

test('db has method process ', function (t) {
  var db = Db('/tmp/test.offset', '/tmp/test.sqlite')
  t.equal(typeof (db.process), 'function')
  t.end()
})

test('create throws when paths are not strings', function (t) {
  t.throws(function () {
    Db(null, '', () => {})
  })
  t.throws(function () {
    Db('', null, () => {})
  })
  t.end()
})

test('indexing does not happen until triggered by a call to process', function (t) {
  // TODO: these offset are specific to Piet's log. refactor test to use flume properly.
  var offset = 5754
  var offset2 = 12130
  var offset3 = 18607

  var logPath = '/tmp/test_indexing.sqlite'
  rimraf.sync(logPath)
  var db = Db('/home/piet/.ssb/flume/log.offset', logPath)

  t.equals(db.getLatest(), 0)

  db.process({ chunkSize: 10 })
  t.ok(db.getLatest() === offset)
  db.process({ chunkSize: 10 })
  t.ok(db.getLatest() === offset2)
  db.process({ chunkSize: 10 })
  t.ok(db.getLatest() === offset3)

  t.end()
})

test('a simple query', function (t) {
  t.end()
})

test('can query even when view is behing log', function (t) {
  t.end()
})

test('query will only call back when up to date with provided sequence', function (t) {
  t.end()
})

test('progress', function (t) {
  t.end()
})

test('can check db integrity', function (t) {
  t.end()
})
