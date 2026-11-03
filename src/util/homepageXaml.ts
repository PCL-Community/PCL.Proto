import { xml2js, js2xml, type Element } from 'xml-js'

export const homepagePlaceholders = [
  'user',
  'uuid',
  'login',
  'name',
  'version',
  'version_path',
  'verpath',
  'minecraft',
  'pcl_version',
  'date',
  'time',
  'hint',
]

export const homepageLimit = 2 * 1024 * 1024
export const homepageControls = [
  'StackPanel',
  'WrapPanel',
  'Grid',
  'ColumnDefinition',
  'RowDefinition',
  'Border',
  'TextBlock',
  'Run',
  'Span',
  'Bold',
  'Italic',
  'Underline',
  'LineBreak',
  'Hyperlink',
  'Image',
  'Path',
  'MyCard',
  'MyHint',
  'MyButton',
  'MyTextButton',
  'MyListItem',
  'MyImage',
  'MyIconButton',
  'MyIconTextButton',
]
export const homepageEvents: Record<string, string> = {
  打开网页: 'OpenUrl',
  复制文本: 'CopyText',
  弹出窗口: 'ShowDialog',
  弹出提示: 'ShowHint',
  刷新主页: 'RefreshHomepage',
  刷新页面: 'RefreshPage',
  切换页面: 'SwitchPage',
  今日人品: 'DailyFortune',
  启动游戏: 'LaunchGame',
}
export const supportedHomepageEvents = new Set(['None', ...Object.values(homepageEvents)])

export function controlName(node: Element) {
  return (node.name?.split(':').at(-1) || '').replace(
    /^P(?=Card|Hint|Button|ListItem|Image$)/,
    'My',
  )
}

export function eventOf(node: Element) {
  const attribute = (key: string) => {
    const value = node.attributes?.[`local:CustomEventService.${key}`] ?? node.attributes?.[key]
    const property = node.elements?.find((child) =>
      child.name?.endsWith(`CustomEventService.${key}`),
    )
    return String(value ?? property?.elements?.find((child) => child.type === 'text')?.text ?? '')
  }
  const raw = attribute('EventType')
  const type =
    homepageEvents[raw] ||
    [...supportedHomepageEvents].find((name) => name.toLowerCase() === raw.toLowerCase()) ||
    raw
  return { type, data: attribute('EventData') }
}

export function parseHomepage(xaml: string): { nodes: Element[]; warnings: string[] } {
  if (new TextEncoder().encode(xaml).length > homepageLimit)
    throw new Error('Homepage exceeds 2 MiB')
  if (/<!DOCTYPE|<!ENTITY/i.test(xaml)) throw new Error('DTD and entities are not supported')
  const document = xml2js(
    `<StackPanel>${xaml.replace(/^\uFEFF/, '').replace(/<\?xml[^?]*\?>/g, '')}</StackPanel>`,
    { compact: false },
  ) as Element
  const nodes = document.elements?.[0]?.elements || []
  const warnings = new Set<string>()
  let count = 0
  function visit(node: Element, depth: number) {
    if (++count > 5000 || depth > 64)
      throw new Error('Homepage is too complex (5000 nodes / 64 levels)')
    if (node.type === 'element') {
      const name = controlName(node)
      if (
        !homepageControls.includes(name) &&
        !/\.(?:ColumnDefinitions|RowDefinitions|Children|Content|Inlines|Child)$/.test(name) &&
        !name.includes('CustomEventService.')
      )
        warnings.add(node.name || name)
      const event = eventOf(node)
      if (event.type && !supportedHomepageEvents.has(event.type))
        warnings.add(`EventType: ${event.type}`)
      for (const value of Object.values(node.attributes || {})) {
        if (
          typeof value === 'string' &&
          value.startsWith('{') &&
          !/^\{(?:DynamicResource|StaticResource) [\w.]+\}$/.test(value) &&
          !homepagePlaceholders.some((key) => value.startsWith(`{${key}}`))
        )
          warnings.add(value)
      }
    }
    node.elements?.forEach((child) => visit(child, depth + 1))
  }
  nodes.forEach((node) => visit(node, 0))
  return { nodes, warnings: [...warnings] }
}

export function serializeHomepage(nodes: Element[]) {
  const preserveText = (node: Element): Element => ({
    ...node,
    // xml-js desanitizes text before escaping it; protect literal entity spellings.
    text: node.type === 'text' ? String(node.text ?? '').replaceAll('&', '&amp;') : node.text,
    elements: node.elements?.map(preserveText),
  })
  return js2xml(
    { elements: nodes.map(preserveText) },
    {
      compact: false,
      spaces: 2,
      attributesFn: (attributes) =>
        Object.fromEntries(
          Object.entries(attributes).map(([key, value]) => [
            key,
            String(value)
              .replaceAll('&', '&amp;')
              .replaceAll('<', '&lt;')
              .replaceAll('\n', '&#xA;')
              .replaceAll('\r', '&#xD;'),
          ]),
        ),
    },
  )
}

export function homepageUrl(value: unknown): string | undefined {
  if (typeof value !== 'string') return undefined
  try {
    const url = new URL(value)
    if (['http:', 'https:'].includes(url.protocol) && !url.username && !url.password)
      return url.href
  } catch {
    /* XAML also accepts local/pack paths, resolved separately by the renderer. */
  }
  return undefined
}

export function xamlColor(value: unknown): string | undefined {
  if (typeof value !== 'string') return undefined
  const resource = /^\{(?:DynamicResource|StaticResource) ColorBrush([1-8])\}$/.exec(value)
  if (resource) return `var(--ce-color-${resource[1]})`
  const argb = /^#([\da-f]{2})([\da-f]{6})$/i.exec(value)
  if (argb) return `#${argb[2]}${argb[1]}`
  return /^(?:#[\da-f]{3,6}|[a-z]+)$/i.test(value) ? value : undefined
}

export function homepageRoute(data: string): string {
  const [pageValue = '', subValue = 'Default'] = data.split('|')
  const pages: [string, string][] = [
    ['Launch', '/home'],
    ['Download', '/download'],
    ['Setup', '/setup'],
    ['Tools', '/tools'],
    ['', ''],
    ['InstanceSelect', '/instance_select'],
    ['TaskManager', '/downloading'],
    ['InstanceSetup', '/instance_setting'],
    ['', ''],
    ['', ''],
    ['GameLog', '/logs'],
  ]
  const page = /^\d+$/.test(pageValue)
    ? pages[Number(pageValue)]
    : pages.find(([name]) => name.toLowerCase() === pageValue.toLowerCase())
  if (!page?.[1]) throw new Error(`Unsupported PageType: ${pageValue}`)
  if (!subValue || subValue.toLowerCase() === 'default' || subValue === '0') return page[1]
  const download = [
    ['Default', 'game'],
    ['DownloadInstall', 'game'],
    ['DownloadMod', 'mod'],
    ['DownloadPack', 'modpack'],
    ['DownloadDataPack', 'datapack'],
    ['DownloadResourcePack', 'resourcepack'],
    ['DownloadShader', 'shader'],
    ['DownloadWorld', 'world'],
    ['DownloadCompFavorites', 'favorites'],
    ['DownloadClient', 'client'],
    ['DownloadOptiFine', 'optifine'],
    ['DownloadForge', 'forge'],
    ['DownloadNeoForge', 'neoforge'],
    ['DownloadCleanroom', 'cleanroom'],
    ['DownloadFabric', 'fabric'],
    ['', ''],
    ['DownloadLiteLoader', 'liteloader'],
    ['DownloadLabyMod', 'labymod'],
    ['DownloadLegacyFabric', 'legacyfabric'],
  ]
  const subpages: Record<string, string[][]> = {
    Download: download,
    Setup: [
      ['SetupLaunch', 'launch'],
      ['SetupUI', 'personalization'],
      ['SetupGameManage', 'manage'],
      ['SetupLink', 'link'],
      ['SetupAbout', 'about'],
      ['SetupLog', 'log'],
      ['SetupFeedback', 'feedback'],
      ['SetupGameLink', 'link'],
      ['SetupUpdate', 'update'],
      ['SetupJava', 'java'],
      ['SetupLauncherMisc', 'misc'],
      ['SetupLauncherLanguage', 'language'],
    ],
    Tools: [
      ['Default', 'lobby'],
      ['ToolsGameLink', 'lobby'],
      ['', ''],
      ['ToolsTest', 'wonderbox'],
    ],
    InstanceSetup: [
      ['VersionOverall', 'overview'],
      ['VersionSetup', 'setting'],
      ['VersionExport', 'export'],
      ['VersionWorld', 'save'],
      ['VersionScreenshot', 'screenshot'],
      ['VersionMod', 'mod'],
      ['VersionModDisabled', 'mod/disabled'],
      ['VersionResourcePack', 'resourcepack'],
      ['VersionShader', 'shader'],
      ['VersionSchematic', 'schematic'],
      ['VersionInstall', 'modify'],
      ['VersionServer', 'server'],
      ['VersionModJarInJar', 'mod/jarinjar'],
    ],
  }
  const choices = subpages[page[0]]
  const subpage = /^\d+$/.test(subValue)
    ? choices?.[Number(subValue)]
    : choices?.find(([name]) => name!.toLowerCase() === subValue.toLowerCase())
  if (!subpage?.[1]) throw new Error(`Unsupported PageSubType: ${subValue}`)
  return `${page[1]}/${subpage[1]}`
}

export function homepageText(value: unknown, variables: Record<string, string> = {}) {
  return String(value ?? '').replace(
    /\{([a-z_]+)\}/g,
    (match, key: string) => variables[key] ?? match,
  )
}
