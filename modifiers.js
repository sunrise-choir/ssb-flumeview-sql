const messages = 'messages'
const links = 'links'
const keys = 'keys'
const messageType = 'content_type'
const isDecrypted = 'is_decrypted'

//Get tip of a feed
//Get all replies to a message
//
module.exports.modifiers = {
  whereMessageType: function (query, type) {
    query.where(
      `${messages}.${messageType}`, type
    )
  },
  whereMessageIsNotType: function (query, type) {
    query.whereNot(
      `${messages}.${messageType}`, type
    )
  },
  whereMessageIsPrivate: function (query) {
    query.where(
      `${messages}.${isDecrypted}`, 1
    )
  },
  whereMessageIsNotPrivate: function (query) {
    query.whereNot(
      `${messages}.${isDecrypted}`, 1
    )
  }
}

module.exports.strings = {
  messages,
  links,
  keys
}
