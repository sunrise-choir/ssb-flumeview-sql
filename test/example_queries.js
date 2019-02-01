var test = require('tape')
var config = require('ssb-config')
var { join } = require('path')

var Db = require('../')

var rimraf = require('rimraf')

var logPath = join(config.path, 'flume', 'log.offset')
var secretKey = Buffer.from('')
var publicKey = '@U5GvOKP/YUza9k53DSXxT0mk3PIrnyAmessvNfZl5E0=.ed25519'

function createTestDb (logPath, dbPath) {
  rimraf.sync(dbPath)
  var db = Db(logPath, dbPath, secretKey, publicKey)
  db.process({ chunkSize: 10000 })
  return db
}

var db = createTestDb(logPath, '/tmp/example_queries.sqlite3')

test('content of my most recent 20 posts', function (t) {
  db.knex
    .select(['content', 'author'])
    .from('messages_raw')
    .join('authors', 'messages_raw.author_id', 'authors.id')
    .where('authors.is_me', 1)
    .orderBy('flume_seq', 'desc')
    .limit(20)
    .asCallback(function (err, results) {
      t.error(err)
      t.equal(results.length, 20)
      t.ok(results.every(function (result) {
        return result.author === publicKey
      }))

      t.end()
    })
})

test('all the votes on this message', function (t) {
  var messageKey = '%WCQziqKuknTZvaPgl0JR0hMmR5GQ+fhJyu9BZpW95wI=.sha256'
  db.knex
    .select()
    .from('links')
    .join('messages_raw', 'messages_raw.key_id', 'links.link_from_key_id')
    .where('links.link_to_key', messageKey)
    .where('content_type', 'vote')
    .asCallback(function (err, results) {
      t.error(err)
      t.equal(results.length, 1)
      t.end()
    })
})

test.onFinish(function () {
  db.knex.destroy()
})
