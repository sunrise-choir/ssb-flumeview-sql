'use strict'
var Push = require('pull-pushable')
var Obv = require('obv')

var binding = require('./build/Release/binding.node')

module.exports = function SsbDb (path, since, cb) {
  binding.init(path, function (err, db) {
    if (err) return cb(err)

    var updated = Obv()

    // TODO: Things still to work out:
    // - maybe db init shouldn't be async
    // - query should work asap, even if there's indexing to do.
    // - progress / status
    // - ric + opts.chunkSize for background processing?

    // TODO: this still has a one to one coupling between appends to the log and view updates.
    since(function (seq) {
      db.update(seq, function (err) {
        if (err) throw err
        updated.set(seq)
      })
    })

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

    cb(null, {
      query
    })
  })
}
