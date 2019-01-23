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
  backLinksReferences
}

module.exports.strings = {
  messages,
  links,
  keys
}

function whereMessageType (query, type) {
  query.where(
    messageType, type
  )
}

function whereMessageIsNotType (query, type) {
  query.whereNot(
    messageType, type
  )
}
function whereMessageIsNotRoot (query, id) {
  query.whereNot(
    messageRoot,
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

function backLinksReferences (query, id, knex) {
  query
    .modify(whereMessageIsNotType, 'about')
    .modify(whereMessageIsNotType, 'vote')
    .modify(whereMessageIsNotType, 'tag')
    .modify(whereMessageIsNotRoot, id)
    .join(links, 'links.link_to', key)
    .where(
      'links.link_to',
      id
    )
}
