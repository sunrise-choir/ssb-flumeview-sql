'use strict'

var binding = require('./build/Release/binding.node')

function Message (key, ts, previous, author, sequence, timestamp, hash, content, signature) {
  this.key = key
  this.value = new Value(previous, author, sequence, timestamp, hash, content, signature)
  this.timestamp = ts
}

function Value (previous, author, sequence, timestamp, hash, content, signature) {
  this.previous = previous
  this.author = author
  this.sequence = sequence
  this.timestamp = timestamp
  this.hash = hash
  this.content = content
  this.signature = signature
}

module.exports = {
  parseJsonWithConstructor: function(string) {
    return binding.parseJsonWithConstructor(string, Message)
  },
  parseJsonAsync: function(string,cb) {
    binding.parseJsonAsync(Buffer.from(string), Message, cb)
  },
  parseCborWithConstructor: function(string) {
    return binding.parseCborWithConstructor(string, Message)
  },
  parseJson: binding.parseJson,
  parseCbor: binding.parseCbor,
  toCbor: binding.toCbor,
  toJson: binding.toJson
}

