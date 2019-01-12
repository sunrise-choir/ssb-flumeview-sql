'use strict'

var Knex = require('knex')
var SqlView = require('./build/Release/binding.node')

module.exports = function SsbDb (logPath, dbPath, since) {
  if (typeof (logPath) !== 'string') {
    throw new TypeError('Expected logPath to be a string')
  }
  if (typeof (dbPath) !== 'string') {
    throw new TypeError('Expected dbPath to be a string')
  }
  if (typeof (since) !== 'function') {
    throw new TypeError("Expected since observable to be a function. Normally this is the 'since' obeservable from flumedb.")
  }

  var knex = Knex({
    client: 'sqlite3',
    connection: {
      filename: dbPath
    }
  })

  var db = new SqlView(logPath, dbPath)

  var exports = {
    process,
    getLatest: () => db.getLatest(),
    knex
  }

  return exports

  function process (opts) {
    opts = opts || { chunkSize: -1 }
    db.process(opts.chunkSize)
  }

  // queryBuilder: (knex) -> knexQuery

  function query ({ query, whenUpToSequence }, cb) {
    if (typeof (query) !== 'string') {
      throw new TypeError('Expected query to be a string')
    }
    if (typeof (cb) !== 'function') {
      throw new TypeError('Expected cb to be a function')
    }

    function doQuery () {
      db.query(query, function (err, result) {
        // db.query returns a string of json data. We need to parse it.
        cb(err, JSON.parse(result))
      })
    }

    if (whenUpToSequence) {
      var remove

      remove = since(function (latest) {
        if (latest >= whenUpToSequence) {
          // unsubscribe from the obv.
          remove()

          // do the query
          doQuery()
        }
      })
    } else {
      doQuery()
    }
  }
}
