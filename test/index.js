var test = require('tape')
var Db = require('../')

test.skip('create', function (t) {
  var db = Db('/tmp/test.offset', '/tmp/test.sqlite')
  t.ok(db)
  t.end()
})

test.skip('db has function getLatest ', function (t) {
  var db = Db('/tmp/test.offset', '/tmp/test.sqlite')
  t.equal(typeof (db.getLatest), 'function')
  t.end()
})

test.skip('db has function query ', function (t) {
  var db = Db('/tmp/test.offset', '/tmp/test.sqlite')
  t.equal(typeof (db.query), 'function')
  t.end()
})

test.skip('db has function process ', function (t) {
  var db = Db('/tmp/test.offset', '/tmp/test.sqlite')
  t.equal(typeof (db.process), 'function')
  t.end()
})

test('create throws when paths are not strings', function (t) {
  t.throws(function () {
    Db(null, '')
  })
  t.throws(function () {
    Db('', null)
  })
  t.end()
})

test('indexing does not happen until triggered by a call to update', function (t) {
  t.end()
})

test('update only indexes chunks of a given size', function (t) {
  t.end()
})

test('a simple query', function (t) {
  t.end()
})

test('a live query only updates when since emits a new value, and only if that value is different to last', function (t) {
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
