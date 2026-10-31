import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import crypto from 'node:crypto'
import { pclIcons } from '../src/components/icons/pclIcons.ts'
import {
  classifyVersion,
  getMinecraftVersions,
  getMinecraftChangelogUrl,
} from '../src/api/gameVersions.ts'
import modrinthApi from '../src/api/modrinthApi.ts'
import { pclImagePath } from '../src/api/pclAssets.ts'
import {
  installWarnings,
  loaderVisible,
  selectLoader,
  fabricInstanceName,
  instanceDefaultName,
  loaderConflict,
  versionDrop,
} from '../src/api/installSelection.ts'
import { maxLogLines } from '../src/util/logLines.ts'
const read = (path) => JSON.parse(fs.readFileSync(new URL(path, import.meta.url), 'utf8'))
const findNode = (node, name) =>
  node.attrs.Name === name
    ? node
    : node.children?.map((child) => findNode(child, name)).find(Boolean)
const manifest = read('../src/assets/pcl-manifest.json')
const strings = {
  ...read('../src/assets/languages/zh-CN.json'),
  ...read('../src/assets/messages/zh-CN.json'),
}
const iconNames = new Set(read('../src/assets/icon-names.json'))
const navActions = read('../src/assets/nav-actions.json')
assert.equal(navActions.PageSetupLeft.ItemLaunch.kind, 'initialize')
assert.equal(navActions.PageSetupLeft.ItemFeedback.kind, 'refresh')
assert.equal(navActions.PageDownloadLeft.ItemMod.icon, 'lucide/refresh-cw')
for (const name of iconNames)
  assert.ok(pclIcons[`lucide/${name}`], `missing package component: ${name}`)
const used = new Set()
const icons = new Set()
const types = new Set()
for (const page of Object.keys(manifest.pages)) {
  const tree = read(`../src/assets/layouts/${page}.json`)
  const visit = (node) => {
    types.add(node.type)
    for (const [key, value] of Object.entries(node.attrs)) {
      for (const match of value.matchAll(/\{(?:Static|Dynamic)Resource ([^}]+)\}/g)) {
        if (
          /^(Common|Launch|Main|Download|Instance|Setup|Tools|Select|Speed|Log|Help)\./.test(
            match[1],
          )
        ) {
          assert.ok(strings[match[1]] !== undefined, `${page}: missing text ${match[1]}`)
          used.add(match[1])
        }
      }
      if ((key === 'SvgIcon' || key === 'Icon') && value.startsWith('lucide/')) {
        assert.ok(
          iconNames.has(value.replace('lucide/', '')),
          `${page}: missing Lucide component ${value}`,
        )
        icons.add(value)
      }
    }
    node.children?.forEach(visit)
  }
  visit(tree)
}
const exportPage = read('../src/assets/layouts/PageInstanceExport.json')
assert.equal(
  findNode(exportPage, 'CheckOptionsBasic').children[0].children[0].attrs.DefaultChecked,
  'True',
)
assert.ok(findNode(read('../src/assets/layouts/PageSetupLaunch.json'), 'ComboArgumentPriority'))
assert.ok(findNode(read('../src/assets/layouts/PageSetupUI.json'), 'CardSwitch'))
const paths = read('../src/assets/vector-paths.json')
for (const [name, item] of Object.entries(paths)) {
  const svg = fs.readFileSync(
    new URL(`../public/pcl/geometry/${name}.svg`, import.meta.url),
    'utf8',
  )
  assert.ok(svg.includes(item.viewBox), `wrong geometry bounds: ${name}`)
  assert.ok(!svg.includes('d="F1'), 'WPF fill-rule prefix must not be part of the SVG path')
  assert.ok(
    item.viewBox.split(' ').every((value) => Number.isFinite(Number(value))),
    `invalid geometry bounds: ${name}`,
  )
  if (item.style.stroke)
    assert.ok(svg.includes(`stroke="${item.style.stroke}"`), `lost stroke: ${name}`)
}
const sourcePaths = read('../src/assets/control-geometry.json')
const logo = Object.values(paths).find(
  (item) => item.data === sourcePaths['FormMain.ShapeTitleLogo'],
)
assert.equal(logo.style.fill, 'none')
assert.equal(logo.style['stroke-width'], '2.2')
const titleLogo = fs.readFileSync(
  new URL('../src/assets/icons/TitleLogo.svg', import.meta.url),
  'utf8',
)
assert.match(
  titleLogo,
  /width="52" height="16" viewBox="0 0 52 16"/,
  'retain the original Proto title logo proportions',
)

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
const root = new URL('../', import.meta.url)
const walk = (dir) =>
  fs
    .readdirSync(dir, { withFileTypes: true })
    .flatMap((entry) =>
      entry.isDirectory() ? walk(path.join(dir, entry.name)) : [path.join(dir, entry.name)],
    )
const files = walk(fileURLToPath(new URL('../src', import.meta.url))).filter((file) =>
  /\.(vue|tsx?)$/.test(file),
)
const names = new Set(files.map((file) => path.basename(file).replace(/\.[^.]+$/, '')))
for (const name of names)
  assert.ok(!/^(My|Ce)[A-Z]/.test(name), `unmigrated component name: ${name}`)
for (const page of Object.keys(manifest.pages))
  assert.ok(page === 'FormMain' || names.has(page), `missing Vue page or control: ${page}`)
for (const file of files) {
  const source = fs.readFileSync(file, 'utf8')
  const imports = source.replace(/<!--[\s\S]*?-->|\/\*[\s\S]*?\*\/|^\s*\/\/.*$/gm, '')
  for (const match of imports.matchAll(
    /(?:\bfrom\s*|\bimport\s*(?:\(\s*)?)['"]((?:@\/|\.{1,2}\/)[^'"]+)['"]/g,
  )) {
    const specifier = match[1].split('?')[0]
    const target = specifier.startsWith('@/')
      ? path.join(fileURLToPath(new URL('../src', import.meta.url)), specifier.slice(2))
      : path.resolve(path.dirname(file), specifier)
    const resolved = ['', '.ts', '.tsx', '.js', '.vue', '/index.ts']
      .map((suffix) => target + suffix)
      .find((candidate) => fs.existsSync(candidate) && fs.statSync(candidate).isFile())
    assert.ok(resolved, `${file}: unresolved import ${specifier}`)
    let parent = path.parse(resolved).root
    for (const segment of resolved.slice(parent.length).split(path.sep)) {
      assert.ok(
        fs.readdirSync(parent).includes(segment),
        `${file}: incorrect import casing ${specifier}`,
      )
      parent = path.join(parent, segment)
    }
  }
  assert.ok(
    !/from\s+['"][^'"]*assets\/layouts\//.test(source),
    `runtime XAML layout interpreter: ${file}`,
  )
  assert.ok(!/:label="text\(''\)"/.test(source), `empty checkbox label: ${file}`)
  for (const match of source.matchAll(
    /(?:\btext|\bpclText)\('((?:Common|Launch|Main|Download|Instance|Setup|Tools|Select|Speed|LogPage|Help|Application|Proto)\.[^']+)'/g,
  )) {
    assert.ok(strings[match[1]] !== undefined, `${file}: missing UI text ${match[1]}`)
  }
  for (const match of source.matchAll(/['"](lucide\/[a-z0-9-]+)['"]/g))
    assert.ok(pclIcons[match[1]], `${file}: unknown icon ${match[1]}`)
  for (const match of source.matchAll(/\bimage\('([^']+)'/g)) {
    if (/^https?:/.test(match[1])) continue
    const asset = pclImagePath(match[1], '')
    assert.ok(fs.existsSync(new URL(`public/${asset}`, root)), `${file}: missing image ${asset}`)
  }
}
for (const loader of [
  'Cleanroom',
  'Fabric',
  'Forge',
  'LabyMod',
  'LegacyFabric',
  'LiteLoader',
  'NeoForge',
  'Optifine',
]) {
  for (const key of ['Intro.Title', 'Intro.Description', 'LoadingList'])
    assert.ok(strings[`Download.Version.${loader}.${key}`], `missing loader text: ${loader}.${key}`)
}
const options = read('../src/assets/export-options.json')
const defaults = read('../src/assets/setting-defaults.json')
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
assert.equal(defaults.PageSetupUI.SliderLauncherOpacity, 600)
assert.equal(defaults.PageSetupUI.SliderBlurValue, 16)
assert.equal(defaults.PageSetupUI.SliderBlurSamplingRate, 70)
assert.equal(defaults.PageSetupLaunch.CheckAdvanceRunWait, true)
assert.equal(defaults.PageInstanceSetup.CheckAdvanceRunWait, true)
assert.equal(defaults.PageInstanceSetup.ComboArgumentIndieV2, '1')
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
assert.equal(
  options.length,
  new Set(options.map((option) => option.id)).size,
  'export option IDs must be unique',
)
assert.ok(options.find((option) => option.id === 'CheckOptionsBasic').checked)
assert.ok(
  options.every((option) => option.title && strings[option.title.match(/Resource (.+)}/)[1]]),
)
const header = fs.readFileSync(
  new URL('../src/components/WindowHeader.vue', import.meta.url),
  'utf8',
)
assert.ok(header.includes('.title-tag Proto'), 'retain the Proto badge')
const html = fs.readFileSync(new URL('../index.html', import.meta.url), 'utf8')
assert.match(
  html,
  /<link\b[^>]*rel="icon"[^>]*href="\/PCL\.Proto\.svg"/,
  'use the current app logo as the website favicon',
)
const about = fs.readFileSync(
  new URL('../src/views/Setup/PageSetupAbout.vue', import.meta.url),
  'utf8',
)
for (const identity of ['AMagicPear', 'Vue.js', 'Rust Programming Language', 'Tauri', 'PCL.Proto'])
  assert.ok(
    about.replace(/pclText\('([^']+)'\)/g, (all, key) => strings[key] ?? all).includes(identity),
    `retain Proto identity: ${identity}`,
  )
if (process.argv[2])
  for (const [name, item] of Object.entries(manifest.pages)) {
    assert.equal(
      crypto
        .createHash('sha256')
        .update(fs.readFileSync(path.join(process.argv[2], item.path)))
        .digest('hex'),
      item.sha256,
      `source snapshot changed: ${name}`,
    )
  }
console.log(
  `UI contract: ${Object.keys(manifest.pages).length} source layouts, ${used.size} layout text keys, ${files.length} frontend files, ${icons.size} referenced Lucide icons, ${Object.keys(paths).length} bounded geometries checked.`,
)
