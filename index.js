'use strict'

var binding = require('./build/Release/binding.node')

module.exports.decrypt = function decrypt (cypher, secret, cb) {
  if (cb) {
    binding.decryptAsync(cypher, secret, cb)
  } else {
    return binding.decrypt(cypher, secret)
  }
}
