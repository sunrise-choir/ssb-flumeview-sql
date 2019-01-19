const links = 'links'
const keys = 'keys'
const authors = 'authors'

const messages = 'messages'
const keyId = `${messages}.key_id`
const authorId = `${messages}.author_id`
const messageType = `${messages}.content_type`
const messageRootId = `${messages}.root_id`
const isDecrypted = `${messages}.is_decrypted`

// Get tip of a feed
// Get all replies to a message
//
module.exports.modifiers = {
  whereMessageType,
  whereMessageIsNotType,
  whereMessageIsPrivate,
  whereMessageIsNotPrivate,
  joinMessagesAuthor,
  joinMessagesKey,
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
function whereMessageIsNotRoot (query, id, knex) {
  query.whereNot(
    messageRootId,
    knex.select('keys.id').from(keys).where('keys.key', id)
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

function joinMessagesAuthor (query) {
  query.join(authors, 'authors.id', authorId)
}

function joinMessagesKey (query) {
  query.join(keys, 'keys.id', keyId)
}

function joinLinksFrom (query) {
  query.join(links, 'links.link_from_id', keyId)
}

function backLinksReferences (query, id, knex) {
  query
    .modify(joinMessagesKey)
    .modify(joinMessagesAuthor)
    .modify(joinLinksFrom)
    .modify(whereMessageIsNotType, 'about')
    .modify(whereMessageIsNotType, 'vote')
    .modify(whereMessageIsNotType, 'tag')
    .modify(whereMessageIsNotRoot, id, knex)
    .where(
      'links.link_to_id',
      '=',
      knex.select('keys.id').from(keys).where('keys.key', id)
    )
}
