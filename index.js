'use strict'

var Knex = require('knex')
var SqlView = require('./build/Release/binding.node')

module.exports = function SsbDb (logPath, dbPath, secretKey, pubKey) {
  if (typeof (logPath) !== 'string') {
    throw new TypeError('Expected logPath to be a string')
  }
  if (typeof (dbPath) !== 'string') {
    throw new TypeError('Expected dbPath to be a string')
  }
  if (!Buffer.isBuffer(secretKey)) {
    throw new TypeError('Expected secret key to be a buffer. This should be the secret key returned by ssb-keys.')
  }
  if (typeof (pubKey) !== 'string') {
    throw new TypeError('Expected pubKey to be a string')
  }

  var knex = Knex({
    client: 'sqlite3',
    useNullAsDefault: true,
    connection: {
      filename: dbPath
    }
  })

  var db = new SqlView(logPath, dbPath, secretKey)

  var exports = {
    process,
    getLatest: () => db.getLatest(),
    knex,
    modifiers: require('./modifiers').modifiers,
    strings: require('./modifiers').strings
  }

  return exports

  function process (opts) {
    opts = opts || { chunkSize: -1 }
    db.process(opts.chunkSize)
  }
}

module.exports.modifiers = require('./modifiers').modifiers
module.exports.strings = require('./modifiers').strings
