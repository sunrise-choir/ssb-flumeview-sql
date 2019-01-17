var test = require('tape')
var Db = require('../')
var rimraf = require('rimraf')
var { messages, links, keys } = require('../modifiers').strings
var {
  whereMessageType,
  whereMessageIsNotType,
  whereMessageIsPrivate,
  whereMessageIsNotPrivate,
  joinMessagesAuthor,
  joinMessagesKey,
  joinLinksFrom,
  backLinksReferences
} = require('../modifiers').modifiers

const numRows = 100

function createTestDB () {
  var logPath = '/tmp/test_indexing.sqlite'
  var secretKey = Buffer.from('')
  rimraf.sync(logPath)
  var db = Db('/home/piet/.ssb/flume/log.offset', logPath, secretKey)

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

test('join author to messages', function (t) {
  var db = createTestDB()

  db
    .knex(messages)
    .modify(joinMessagesAuthor)
    .limit(1)
    .then(function (result) {
      t.ok(result[0].author) // We don't have a secret key set, so there will be 0 private messages
      t.end()
      db.knex.destroy()
    })
})

test('join message key to message', function (t) {
  var db = createTestDB()
  var key = '%/v5mCnV/kmnVtnF3zXtD4tbzoEQo4kRq/0d/bgxP1WI=.sha256'

  db
    .knex(messages)
    .modify(joinMessagesKey)
    .limit(1)
    .then(function (result) {
      t.equal(result[0].key, key) // We don't have a secret key set, so there will be 0 private messages
      t.end()
      db.knex.destroy()
    })
})

test('join message key and authors to message', function (t) {
  var db = createTestDB()
  var key = '%/v5mCnV/kmnVtnF3zXtD4tbzoEQo4kRq/0d/bgxP1WI=.sha256'
  var author = '@U5GvOKP/YUza9k53DSXxT0mk3PIrnyAmessvNfZl5E0=.ed25519'

  db
    .knex(messages)
    .modify(joinMessagesKey)
    .modify(joinMessagesAuthor)
    .limit(1)
    .then(function (result) {
      t.equal(result[0].key, key)
      t.equal(result[0].author, author)
      t.end()
      db.knex.destroy()
    })
})

test('backlinks', function (t) {
  var id = '&MxwsfZoq7X6oqnEX/TWIlAqd6S+jsUA6T1hqZYdl7RM=.sha256'
  var resultKey = '%kLWDux4wCG+OdQWAHnpBGzGlCehqMLfgLbzlKCvgesU=.sha256'
  var db = createTestDB()
  // give me the message.key, message.author and message.timestamp of all the messages that link_to this key.
  db
    .knex
    .select([
      'keys.key as key',
      'authors.author as author',
      'messages.received_time as timestamp'
    ])
    .from(messages)
    .modify(backLinksReferences, id, db.knex)
    .then(function (results) {
      t.equal(results[0].key, resultKey)
      t.end()
      db.knex.destroy()
    })
})
