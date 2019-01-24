const links = 'links'
const keys = 'keys'
const authors = 'authors'

const messages_raw = 'messages_raw'
const messages = 'messages'
const keyId = `${messages}.key_id`
const key = `${messages}.key`
const messageesAuthorId = `${messages}.author_id`
const messageesAuthor = `${messages}.author`
const messageType = `${messages}.content_type`
const messageRootId = `${messages}.root_id`
const messageRoot = `${messages}.root`
const messageFork = `${messages}.fork`
const isDecrypted = `${messages}.is_decrypted`

// Get tip of a feed
// Get all replies to a message
//
module.exports.modifiers = {
  whereMessageType,
  whereMessageIsNotType,
  whereMessageIsPrivate,
  whereMessageIsNotPrivate,
  joinLinksFrom,
  joinLinksTo,
  backLinksReferences
}

module.exports.strings = {
  messages,
  links,
  keys
}

function whereMessageType (query, typeString) {
  query.where(
    messageType, typeString
  )
}

function whereMessageIsNotType (query, typeString) {
  query.whereNot(
    messageType, typeString
  )
}
function whereMessageIsNotRoot (query, id) {
  query.whereNot(
    messageRoot,
    id
  )
}
function whereMessageIsNotFork (query, id) {
  query.whereNot(
    messageFork,
    id
  )
}
function whereMessageIsPrivate (query) {
  query.where(
    isDecrypted, 1
  )
}

function whereMessageIsNotPrivate (query) {
  query.whereNot(
    isDecrypted, 1
  )
}

function joinLinksFrom (query) {
  query.join(links, 'links.link_from', key)
}

function joinMessagesOnLinksFrom (query) {
  query.join(messages, 'links.link_from', key)
}

function joinLinksTo (query) {
  query.join(links, 'links.link_to', key)
}

function backLinksReferences (query, id, knex) {
  query
    .modify(joinMessagesOnLinksFrom)
    .modify(whereMessageIsNotRoot, id)
    .modify(whereMessageIsNotType, 'about')
    .modify(whereMessageIsNotType, 'vote')
    .modify(whereMessageIsNotType, 'tag')
    .where(
      'links.link_to',
      id
    )
}
