var test = require('tape')
var Db = require('../')
var rimraf = require('rimraf')
var { messages, links, keys } = require('../modifiers').strings
var {
  whereMessageType,
  whereMessageIsNotType,
  whereMessageIsPrivate,
  whereMessageIsNotPrivate,
  whereMessageIsNotFork,
  whereMessageIsNotRoot,
  backLinksReferences,
  joinLinksTo,
  joinLinksFrom

} = require('../modifiers').modifiers

const numRows = 1000

function createTestDB (offsetPath) {
  offsetPath = offsetPath || '/home/piet/.ssb/flume/log.offset'
  var logPath = '/tmp/test_modifiers.sqlite'
  var secretKey = Buffer.from('')
  rimraf.sync(logPath)
  var db = Db(offsetPath, logPath, secretKey)

  db.process({ chunkSize: numRows })

  return db
}

test('messagesByType', function (t) {
  var db = createTestDB()

  db
    .knex(messages)
    .modify(whereMessageType, 'idfkjdfsdfdf')
    .then(function (result) {
      t.equal(result.length, 0, 'No messages found of unknown type')
      t.end()
      db.knex.destroy()
    })
})

test('where message is not type', function (t) {
  var db = createTestDB()

  db
    .knex(messages)
    .modify(whereMessageIsNotType, 'idfkjdfsdfdf')
    .limit(10)
    .then(function (result) {
      t.equal(result.length, 10)
      t.end()
      db.knex.destroy()
    })
})

test('private messages', function (t) {
  var db = createTestDB()

  db
    .knex(messages)
    .modify(whereMessageIsPrivate)
    .then(function (result) {
      t.equal(result.length, 0) // We don't have a secret key set, so there will be 0 private messages
      t.end()
      db.knex.destroy()
    })
})

test('not private messages', function (t) {
  var db = createTestDB()

  db
    .knex(messages)
    .modify(whereMessageIsNotPrivate)
    .then(function (result) {
      t.equal(result.length, numRows) // We don't have a secret key set, so there will be 0 private messages
      t.end()
      db.knex.destroy()
    })
})

test.only('get some backlinks', function (t) {
  var db = createTestDB()
  db.process(5000)
  db
    .knex
    .select()
    .from(messages)
    .modify(joinLinksFrom)
    .modify(whereMessageIsNotType, 'about')
    .modify(whereMessageIsNotType, 'vote')
    .modify(whereMessageIsNotType, 'tag')
    .limit(20)
    .offset(10)
    .then(function (results) {
      t.end()
      console.log(results)
      db.knex.destroy()
    })
    .catch(function (err) {
      console.log(err)
      db.knex.destroy()
    })
})

test('backlinks', function (t) {
  var id = '%c2qA4o+aiMzkx0QzV48WZRxv/VaiXXURbHSwFnL9rYo=.sha256'
  var resultKey = '%IH7489JoCxbVcAB9Sn9Y0OpuXJ3/aQwJnPIMbiQimtE=.sha256'
  var db = createTestDB()

  db
    .knex
    .select([
      'links.link_from as key',
      'author',
      'received_time as timestamp'
    ])
    .from(links)
    .modify(backLinksReferences, id, db.knex)
    .then(function (results) {
      t.equal(results[0].key, resultKey)
      t.end()
      db.knex.destroy()
    })
})
