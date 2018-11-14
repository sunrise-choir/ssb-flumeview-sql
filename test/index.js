var test = require('tape')
var {parseLegacy, stringifyLegacy, encodeCbor, parseCbor} = require('../')

var testMessage = require('./simple.json')
var testString = JSON.stringify(testMessage)

test('parses ok', function (t) {
  var actual = parseLegacy(testString)
  t.deepEqual(actual, testMessage)
  t.end()
})

test.skip('parses weird failing thing ok', function (t) {
  var testMessage = require('./weird-failure.json')
  var testString = JSON.stringify(testMessage)
  var actual = parseLegacy(testString)
  t.ok(actual)
  t.end()
})

test('stringify message', function(t) {
  var string = stringifyLegacy(testMessage) 
  t.deepEqual(JSON.parse(string), testMessage)
  t.end()
})

test('encode / decode cbor', function(t) {
  var encodedMessage = encodeCbor(testMessage) 
  var parsedMessage = parseCbor(encodedMessage)

  t.deepEqual(parsedMessage, testMessage)
  t.end()
})
