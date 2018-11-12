'use strict'

var binding = require('./build/Release/binding.node')

function Message (key, previous, author, sequence, hash, content, signature) {
  this.key = key
  this.value = new Value(previous, author, sequence, hash, content, signature)
}

function Value (previous, author, sequence, hash, content, signature) {
  this.previous = previous
  this.author = author
  this.sequence = sequence
  this.hash = hash
  this.content = content
  this.signature = signature
}

module.exports.parseLegacyString = function parseLegacyString (string) {
  return binding.parse_legacy_with_constructor(string, Message)
}
