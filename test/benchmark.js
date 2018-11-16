var pull = require('pull-stream')
var marky = require('marky')

var {toJson, toCbor, parseJson, parseCbor, parseCborWithConstructor, parseJsonWithConstructor} = require('../');
var messages = require('./output.json').queue

var messageStrings = messages.map(JSON.stringify)

pull(
  pull.once(marky),
  pull.map((marky) => {
    marky.mark('JSON.stringify')

    messages.map(JSON.stringify)

    marky.stop('JSON.stringify')
    return marky
  }),
  pull.map((marky) => {
    marky.mark('JSON.parse')

    messageStrings.map(JSON.parse)

    marky.stop('JSON.parse')
    return marky
  }),
  pull.map((marky) => {
    marky.mark('toJson')

    messages.map(toJson)

    marky.stop('toJson')
    return marky
  }),
  pull.map((marky) => {
    marky.mark('parseJson')

    messageStrings.map(parseJson)

    marky.stop('parseJson')
    return marky
  }),
  pull.map((marky) => {
    marky.mark('parseJsonConstructor')

    messageStrings.map(parseJsonWithConstructor)

    marky.stop('parseJsonConstructor')
    return marky
  }),
  pull.map((marky) => {
    marky.mark('encode cbor')

    var cbors = messages.map(toCbor)

    marky.stop('encode cbor')

    marky.mark('parse cbor')

    cbors.map(parseCbor)

    marky.stop('parse cbor')

    marky.mark('parse cbor with constructor')

    cbors.map(parseCborWithConstructor)

    marky.stop('parse cbor with constructor')
    return marky
  }),
  pull.drain((marky) => {
    var entries = marky.getEntries()
    entries.map(entry => {
      console.log(entry.name, entry.duration)
    })
  })
)
