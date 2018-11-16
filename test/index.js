var test = require('tape')
var {parseJson, toJson, toCbor, parseCbor, parseJsonWithConstructor, parseCborWithConstructor} = require('../')

var testMessage = require('./simple.json')
var testString = JSON.stringify(testMessage)

test('parses ok', function (t) {
  var actual = parseJson(testString)
  t.deepEqual(actual, testMessage)
  t.end()
})

test('parses with constructor ok', function (t) {
  var actual = parseJsonWithConstructor(testString)
  t.deepEqual(actual, testMessage)
  t.end()
})
test.skip('parses weird failing thing ok', function (t) {
  var testMessage = require('./weird-failure.json')
  var testString = JSON.stringify(testMessage)
  var actual = parseJson(testString)
  t.ok(actual)
  t.end()
})

test('stringify message', function(t) {
  var string = toJson(testMessage) 
  t.deepEqual(JSON.parse(string), testMessage)
  t.end()
})

test('encode / decode cbor', function(t) {
  var encodedMessage = toCbor(testMessage) 
  var parsedMessage = parseCbor(encodedMessage)

  t.deepEqual(parsedMessage, testMessage)

  parsedMessage = parseCborWithConstructor(encodedMessage)
  t.deepEqual(parsedMessage, testMessage)
  t.end()
})
