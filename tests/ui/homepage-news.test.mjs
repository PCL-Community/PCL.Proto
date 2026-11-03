import assert from 'node:assert/strict'
import { parseHomepageNews } from '../../src/util/homepageNews.ts'

const parse = (results) => parseHomepageNews(JSON.stringify({ result: { results } }))
const [valid, unsafe] = parse([
  {
    title: 'News',
    url: 'https://www.minecraft.net/article/example',
    image: 'https://www.minecraft.net/image.png',
    time: 1700000000,
  },
  {
    title: 'Unsafe',
    url: 'https://minecraft.net.attacker.example/',
    image: 'javascript:alert(1)',
    time: 1e100,
  },
])
assert.equal(valid.url, 'https://www.minecraft.net/article/example')
assert.equal(valid.image, 'https://www.minecraft.net/image.png')
assert.equal(valid.time, 1700000000)
assert.equal(unsafe.url, undefined)
assert.equal(unsafe.image, undefined)
assert.equal(unsafe.time, undefined)
assert.equal(parse([{ title: 'Credential', url: 'https://user@minecraft.net/' }])[0].url, undefined)
assert.deepEqual(parse([{ title: 2 }, null]), [])
assert.throws(() => parseHomepageNews('{"result":{}}'))
console.log('Homepage news response validation and link restrictions passed.')
