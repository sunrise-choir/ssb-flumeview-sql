var test = require('tape')
var Obv = require('obv')
var Db = require('../')
var rimraf = require('rimraf')
var SsbKeys = require('ssb-keys')

var keys = SsbKeys.loadOrCreate('~/.ssb/secret')

test('create', function (t) {
  var since = Obv()
  var db = Db('/tmp/test.offset', '/tmp/test.sqlite', since)
  t.ok(db)
  t.end()
})

test('db has method getLatest ', function (t) {
  var since = Obv()
  var db = Db('/tmp/test.offset', '/tmp/test.sqlite', since)
  t.equal(typeof (db.getLatest()), 'number')
  t.end()
})

test.skip('db has method query ', function (t) {
  var since = Obv()
  var db = Db('/tmp/test.offset', '/tmp/test.sqlite', since)
  t.equal(typeof (db.query), 'function')
  t.end()
})

test('db has method process ', function (t) {
  var since = Obv()
  var db = Db('/tmp/test.offset', '/tmp/test.sqlite', since)
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

test('create throws when since is not a function', function (t) {
  t.throws(function () {
    Db('', '')
  })
  t.end()
})

test('processing the log in chunks works correctly', function (t) {
  var since = Obv()
  // TODO: these offset are specific to Piet's log. refactor test to use flume properly.
  var offset = 5754
  var offset2 = 12130
  var offset3 = 18607

  var logPath = '/tmp/test_indexingg.sqlite'
  rimraf.sync(logPath)
  var db = Db('/home/piet/.ssb/flume/log.offset', logPath, since)

  t.equals(db.getLatest(), 0)

  db.process({ chunkSize: 10 })
  t.ok(db.getLatest() === offset)
  db.process({ chunkSize: 10 })
  t.ok(db.getLatest() === offset2)
  db.process({ chunkSize: 10 })
  t.ok(db.getLatest() === offset3)

  t.end()
})

test('can query even when view is behind log', function (t) {
  var since = Obv()
  var logPath = '/tmp/test_query.sqlite'
  rimraf.sync(logPath)
  var db = Db('/home/piet/.ssb/flume/log.offset', logPath, since)

  db.knex.select()
    .from('message')
    .then(function (res) {
      t.equal(res.length, 0)
      db.process({ chunkSize: 20 })
      return db.knex.select().from('message')
    })
    .then(function (res) {
      t.equal(res.length, 20)
      t.end()
      db.knex.destroy()
    })
    .catch(t.fail)
})

test('query will only call back when up to date with provided sequence', function (t) {
  t.end()
})

test('can check db integrity', function (t) {
  t.end()
})
