var jayson = require('jayson')

var client = jayson.client.tcp({
  port: 9876
})

var batch = new Array(1E3)
  .fill(0)
  .map((_, idx) => {
    return client.request('get_latest', [idx])
  })

console.time('req1')
client.request('get_latest', [1, 1], function (err, error, result) {
  console.timeEnd('req1')
  console.log('done, with result', result, err, error) // 25
})

console.time('req2')
client.request('get_latest', [2, 2], function (err, error, result) {
  console.timeEnd('req2')
  console.log('done, with result', result, err, error) // 25
})

console.time('req3')
client.request('get_latest', [3, 3], function (err, error, result) {
  console.timeEnd('req3')
  console.log('done, with result', result, err, error) // 25
})

console.time('batch')
client.request(batch, function (err, error, result) {
  console.timeEnd('batch')
})
