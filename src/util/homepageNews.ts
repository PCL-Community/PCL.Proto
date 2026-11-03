export interface HomepageNewsItem {
  title: string
  description: string
  author: string
  image?: string
  imageAltText: string
  url?: string
  time?: number
}

function httpUrl(value: unknown, allowedHosts?: string[]): string | undefined {
  if (typeof value !== 'string') return undefined
  try {
    const url = new URL(value)
    if (!['https:', 'http:'].includes(url.protocol) || url.username || url.password)
      return undefined
    if (
      allowedHosts &&
      !allowedHosts.some((host) => url.hostname === host || url.hostname.endsWith(`.${host}`))
    )
      return undefined
    return url.href
  } catch {
    return undefined
  }
}

export function parseHomepageNews(content: string): HomepageNewsItem[] {
  const data: unknown = JSON.parse(content)
  if (!data || typeof data !== 'object' || !('result' in data))
    throw new Error('Invalid news response')
  const result = data.result
  if (
    !result ||
    typeof result !== 'object' ||
    !('results' in result) ||
    !Array.isArray(result.results)
  )
    throw new Error('Invalid news response')
  return result.results.flatMap((item: unknown) => {
    if (!item || typeof item !== 'object' || !('title' in item) || typeof item.title !== 'string')
      return []
    const value = item as Record<string, unknown>
    return [
      {
        title: item.title,
        description: typeof value.description === 'string' ? value.description : '',
        author: typeof value.author === 'string' ? value.author : '',
        image: httpUrl(value.image),
        imageAltText: typeof value.imageAltText === 'string' ? value.imageAltText : '',
        url: httpUrl(value.url, ['minecraft.net', 'minecraft-services.net', 'microsoft.com']),
        time:
          typeof value.time === 'number' &&
          Number.isFinite(value.time) &&
          Math.abs(value.time) <= 8640000000000
            ? value.time
            : undefined,
      },
    ]
  })
}
