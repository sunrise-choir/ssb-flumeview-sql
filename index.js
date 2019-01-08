'use strict'
var Push = require('pull-pushable')
var Obv = require('obv')

var SqlView = require('./build/Release/binding.node')

module.exports = function SsbDb (logPath, dbPath) {
  if (typeof (logPath) !== 'string') {
    throw new TypeError('Expected logPath to be a string')
  }
  if (typeof (dbPath) !== 'string') {
    throw new TypeError('Expected dbPath to be a string')
  }

  var db = new SqlView(logPath, dbPath)

  var exports = {
    process,
    getLatest: () => db.getLatest()
  }

  return exports

  function process (opts) {
    opts = opts || { chunkSize: -1 }
    db.process(opts.chunkSize)
  }

  // TODO: Things still to work out:
  // - query should work asap, even if there's indexing to do.
  // - progress / status

  function query (query, opts) {
    opts = opts || {}

    if (typeof (query) !== 'string') {
      throw new TypeError('Expected query to be a string')
    }

    var p = Push()

    function pushResult (err, result) {
      if (err) throw err // TODO: maybe don't throw here?
      p.push(result)
    }

    if (opts.live) {
      updated(function (seq) {
        db.query(query, pushResult)
      })
    } else {
      db.query(query, pushResult)
    }

    return p
  }
}
