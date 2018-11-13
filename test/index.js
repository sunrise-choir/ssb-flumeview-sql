var test = require('tape')
var {parseLegacyString} = require('../')

var testMessage = require('./simple.json')
var testString = JSON.stringify(testMessage)

test('parses ok', function (t) {
  var actual = parseLegacyString(testString)
  t.ok(actual)
  t.end()
})

test('parses weird failing thing ok', function (t) {
  var testMessage = require('./weird-failure.json')
  var testString = JSON.stringify(testMessage)
  var actual = parseLegacyString(testString)
  t.ok(actual)
  t.end()
})
