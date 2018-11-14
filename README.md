# ssb-json-parse-native

> Node bindings to parse ssb messages

Provides methods to serialize / deserialize ssb-messages. Supports json and cbor serialisation / derserialisation. 
This is is really just a node wrapper around [legacy-msg-data](https://github.com/ssbrs/legacy-msg-data)

## API

```js
var {parseJson, toJson, parseCbor, toCbor} = require('ssb-json-parse-native')
```

### parseJson(jsonString)

Returns a js object. Identical to JSON.parse but slower (for now)

### parseCbor(cborBuffer)

Returns a js object. Must be passed a buffer.

### toJson(messageObject)

Returns a string. Identical to JSON.stringify but slower (for now)

### toCbor(messageObject)

Returns a buffer.

## Install

With [npm](https://npmjs.org/) installed, run

```
$ npm install ssb-json-parse-native
```

## Acknowledgments


## See Also


## License

AGPL3
