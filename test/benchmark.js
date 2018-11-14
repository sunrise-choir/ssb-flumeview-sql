var pull = require('pull-stream')
var marky = require('marky')

var {stringifyLegacy, encodeCbor, parseLegacy, parseCbor} = require('../');
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
    marky.mark('stringifyLegacy')

    messages.map(stringifyLegacy)

    marky.stop('stringifyLegacy')
    return marky
  }),
  pull.map((marky) => {
    marky.mark('parseLegacy')

    messageStrings.map(parseLegacy)

    marky.stop('parseLegacy')
    return marky
  }),
  pull.map((marky) => {
    marky.mark('encode cbor')

    var cbors = messages.map(encodeCbor)

    marky.stop('encode cbor')

    marky.mark('parse cbor')

    cbors.map(parseCbor)

    marky.stop('parse cbor')

    return marky
  }),
  pull.drain((marky) => {
    var entries = marky.getEntries()
    entries.map(entry => {
      console.log(entry.name, entry.duration)
    })
  })
)
