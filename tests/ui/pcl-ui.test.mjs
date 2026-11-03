import assert from 'node:assert/strict'
import {
  classifyVersion,
  getMinecraftVersions,
  getMinecraftChangelogUrl,
} from '../../src/api/gameVersions.ts'
import modrinthApi from '../../src/api/modrinthApi.ts'
import { pclImagePath } from '../../src/api/pclAssets.ts'
import {
  installWarnings,
  loaderVisible,
  selectLoader,
  fabricInstanceName,
  instanceDefaultName,
  loaderConflict,
  versionDrop,
} from '../../src/api/installSelection.ts'
import { maxLogLines } from '../../src/util/logLines.ts'

for (const [type, id, date, category] of [
  ['release', '1.21.1', '2024-08-08', 'release'],
  ['snapshot', '1.2', '2012-03-01', 'snapshot'],
  ['snapshot', '1.21-pre1', '2024-05-30', 'snapshot'],
  ['snapshot', '1.21-rc1', '2024-06-10', 'snapshot'],
  ['snapshot', '1.21.1', '2024-08-08', 'release'],
  ['snapshot', '20w14infinite', '2020-04-01', 'fool'],
  ['snapshot', '25w14craftmine', '2025-04-01', 'fool'],
  ['old_beta', 'b1.7.3', '2011-07-08', 'old'],
])
  assert.equal(
    classifyVersion({ type, id, releaseTime: date }),
    category,
    `wrong version group: ${id}`,
  )
for (const [source, expected] of [
  ['pack://application:,,,/Images/Skins/Steve.png', '/PCL.Proto/pcl/images/Skins/Steve.png'],
  [
    '/Plain Craft Launcher 2;component/Images/Heads/LTCat.jpg',
    '/PCL.Proto/pcl/images/Heads/LTCat.jpg',
  ],
  ['images/Icons/NoIcon.png', '/PCL.Proto/pcl/images/Icons/NoIcon.png'],
  ['https://example.org/avatar.png', 'https://example.org/avatar.png'],
  ['data:image/png;base64,AA==', 'data:image/png;base64,AA=='],
])
  assert.equal(pclImagePath(source, '/PCL.Proto/'), expected)
const originalFetch = globalThis.fetch
for (const [id, locale, host, query] of [
  ['1.21.1', 'zh-CN', 'zh.minecraft.wiki', 'Java版1.21.1'],
  ['25w14craftmine', 'en-US', 'minecraft.wiki', 'Java Edition 25w14craftmine'],
  ['25w14craftmine', 'zh-CN', 'zh.minecraft.wiki', '25w14craftmine'],
  ['b1.7.3', 'zh-TW', 'zh.minecraft.wiki', 'Java版Beta 1.7.3'],
  ['1.21 & test', 'fr-FR', 'fr.minecraft.wiki', 'Java Edition 1.21 & test'],
]) {
  const url = new URL(getMinecraftChangelogUrl(id, locale))
  assert.equal(url.hostname, host)
  assert.equal(url.searchParams.get('search'), query)
  assert.equal(url.searchParams.get('go'), 'Go')
}
try {
  let requested = ''
  globalThis.fetch = async (url) => {
    requested = String(url)
    return new Response(JSON.stringify({ hits: [], total_hits: 0, offset: 0, limit: 20 }))
  }
  await modrinthApi.searchProjects({
    query: 'a & b',
    facets: [['project_type:mod']],
    index: 'downloads',
    limit: 20,
  })
  const request = new URL(requested)
  assert.equal(request.origin, 'https://api.modrinth.com')
  assert.equal(request.searchParams.get('query'), 'a & b')
  assert.deepEqual(JSON.parse(request.searchParams.get('facets')), [['project_type:mod']])
  assert.equal(request.searchParams.get('index'), 'downloads')
  globalThis.fetch = async () => new Response('{}', { status: 503 })
  await assert.rejects(modrinthApi.searchProjects(), /HTTP 503/)
  globalThis.fetch = async () => new Response('{"hits":null}')
  await assert.rejects(modrinthApi.searchProjects(), /格式错误/)
  globalThis.fetch = async () => new Response('{"versions":[{"id":"broken"}]}')
  await assert.rejects(getMinecraftVersions(), /格式错误/)
  globalThis.fetch = async () =>
    new Response('{"versions":[{"id":"1.21.1","type":"release","releaseTime":"2024-08-08"}]}')
  assert.equal((await getMinecraftVersions()).latest.release.id, '1.21.1')
} finally {
  globalThis.fetch = originalFetch
}
for (const [value, lines] of [
  [0, 50],
  [5, 100],
  [6, 150],
  [13, 500],
  [14, 600],
  [28, 2000],
  [29, Infinity],
])
  assert.equal(maxLogLines(value), lines)
for (const [version, drop] of [
  ['1.12.2', 120],
  ['1.21.1-pre1', 210],
  ['26.1', 261],
  ['18w43b', 130],
  ['17w50a', 120],
  ['1.rv-pre1', 90],
  ['3D Shareware v1.34', 140],
  ['unknown', -1],
])
  assert.equal(versionDrop(version), drop)
const selected = { Forge: '47', OptiFine: 'HD', LiteLoader: '1.12' }
assert.equal(instanceDefaultName('1.20.1', { Forge: '47.2.0' }), '1.20.1-Forge_47.2.0')
assert.equal(
  instanceDefaultName('1.12.2', { LiteLoader: '1', OptiFine: 'HD U F5' }),
  '1.12.2-LiteLoader-OptiFine_HD_U_F5',
)
assert.equal(loaderConflict('Forge', { Fabric: '0.16' }, '1.20.1'), 'Fabric')
assert.equal(loaderConflict('LegacyFabric', { LiteLoader: '1' }, '1.12.2'), 'LiteLoader')
assert.equal(loaderConflict('Forge', { OptiFine: 'HD' }, '1.14.3'), 'OptiFine')
assert.equal(loaderConflict('Forge', {}, '1.20.1'), undefined)
selectLoader(selected, 'Fabric', '0.16')
assert.equal(fabricInstanceName('1.20.1', selected), '1.20.1-Fabric_0.16')
assert.equal(fabricInstanceName('1.20.1', { Fabric: '0.4.0+build.123' }), '1.20.1-Fabric_0.4.0.123')
assert.equal(selected.Forge, undefined)
assert.equal(selected.OptiFine, 'HD')
assert.equal(selected.LiteLoader, '1.12')
assert.deepEqual(installWarnings(selected, '1.21.1'), ['FabricApi', 'OptiFabric', 'ModOptiFine'])
assert.deepEqual(installWarnings(selected, '1.14.4'), ['FabricApi', 'OptiFabricOld'])
selectLoader(selected, 'FabricApi', '0.99')
selectLoader(selected, 'OptiFabric', '1.0')
assert.deepEqual(installWarnings(selected, '1.21.1'), ['ModOptiFine'])
selectLoader(selected, 'LegacyFabric', '0.16')
assert.equal(fabricInstanceName('1.12.2', selected), '1.12.2-LegacyFabric_0.16')
assert.equal(selected.FabricApi, undefined)
assert.equal(selected.OptiFabric, undefined)
assert.deepEqual(installWarnings(selected, '1.12.2'), ['LegacyFabricApi', 'LegacyOptiFabric'])
selectLoader(selected, 'LegacyFabric', '')
assert.equal(selected.LegacyFabricApi, undefined)
assert.deepEqual(installWarnings(selected, '1.12.2'), [])
for (const [loader, version, visible] of [
  ['OptiFine', '1.7.1', false],
  ['OptiFine', '1.7.2', true],
  ['Forge', '1.0', false],
  ['Forge', '1.1', true],
  ['Forge', '24w14a', false],
  ['NeoForge', '1.20', false],
  ['NeoForge', '1.20.1', true],
  ['Cleanroom', '1.12.2', true],
  ['Cleanroom', '1.12.1', false],
  ['LiteLoader', '1.5.1', false],
  ['LiteLoader', '1.5.2', true],
  ['LiteLoader', '1.12.2', true],
  ['LiteLoader', '1.12.3', false],
  ['Fabric', '1.13.2', false],
  ['Fabric', '18w43a', false],
  ['Fabric', '18w43b', true],
  ['Fabric', '1.14-pre1', true],
  ['Fabric', '26.1-snapshot-1', true],
  ['LegacyFabric', '1.2.5', false],
  ['LegacyFabric', '1.3', true],
  ['LegacyFabric', '1.13.2', true],
  ['LegacyFabric', '1.14', false],
  ['LabyMod', '1.8.8', false],
  ['LabyMod', '1.8.9', true],
])
  assert.equal(loaderVisible(loader, version, {}), visible, `${loader} visibility for ${version}`)
assert.equal(loaderVisible('FabricApi', '1.21.1', { Fabric: '0.16' }), true)
assert.equal(loaderVisible('OptiFabric', '1.21.1', { Fabric: '0.16' }), false)
assert.equal(loaderVisible('OptiFabric', '1.21.1', { Fabric: '0.16', OptiFine: 'HD' }), true)
console.log('Version parsing, API failures and loader selection passed.')
