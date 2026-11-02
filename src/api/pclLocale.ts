import { computed, readonly, ref, shallowReactive } from 'vue'
import zh from '../assets/languages/zh-CN.json'
import appZh from '../assets/messages/zh-CN.json'
import languageNames from '../assets/language-names.json'

const loaders = import.meta.glob('../assets/languages/*.json', { import: 'default' }) as Record<
  string,
  () => Promise<Record<string, string>>
>
const appLoaders = import.meta.glob('../assets/messages/*.json', { import: 'default' }) as Record<
  string,
  () => Promise<Record<string, string>>
>
export const pclStrings: Record<string, string> = { ...zh, ...appZh }
export const pclLanguages = languageNames
const currentLocale = ref('zh-CN')
export const pclLocale = readonly(currentLocale)
const languagePreference = ref('auto')
export const pclLanguagePreference = readonly(languagePreference)
const formatPreference = ref('auto')
export const pclFormatPreference = readonly(formatPreference)
const systemFormatLocale = ref(Intl.DateTimeFormat().resolvedOptions().locale)
export const pclFormatLocale = computed(() =>
  formatPreference.value === 'auto'
    ? systemFormatLocale.value
    : formatPreference.value === 'ui-language'
      ? currentLocale.value
      : formatPreference.value,
)
const loaded = shallowReactive<Record<string, Record<string, string>>>({ 'zh-CN': pclStrings })
const pending = new Map<string, Promise<void>>()
let languageChange = 0

function normalizeLocale(value: string): string {
  return pclLanguages.find((locale) => locale.toLowerCase() === value.toLowerCase()) || 'zh-CN'
}

export function resolveSystemLanguage(
  language = typeof navigator === 'undefined' ? '' : navigator.language,
): string {
  const supported = ['zh-CN', 'zh-TW', 'en-US', 'en-GB', 'ja-JP', 'fr-FR', 'es-ES']
  const code = language.replaceAll('_', '-').toLowerCase()
  return (
    supported.find((locale) => locale.toLowerCase() === code) ||
    supported.find((locale) => locale.toLowerCase().startsWith(`${code.split('-')[0]}-`)) ||
    'zh-CN'
  )
}

export function setPclFormatLocale(value: string) {
  let normalized = value === 'ui-language' ? value : 'auto'
  if (value !== 'auto' && value !== 'ui-language') {
    try {
      normalized = Intl.getCanonicalLocales(value)[0] || 'auto'
    } catch {
      /* use system format */
    }
  }
  formatPreference.value = normalized
  try {
    localStorage.setItem('pcl-format-locale', normalized)
  } catch (error) {
    console.warn('Unable to save format preference', error)
  }
}

export async function loadPclLocale(locale: string): Promise<void> {
  if (loaded[locale]) return
  const loader = loaders[`../assets/languages/${locale}.json`]
  if (!loader) throw new RangeError(`Unsupported language: ${locale}`)
  let request = pending.get(locale)
  if (!request) {
    request = Promise.all([loader(), appLoaders[`../assets/messages/${locale}.json`]?.()])
      .then(([strings, appStrings]) => {
        loaded[locale] = { ...strings, ...appStrings }
      })
      .finally(() => pending.delete(locale))
    pending.set(locale, request)
  }
  await request
}

export async function setPclLocale(locale: string): Promise<void> {
  const change = ++languageChange
  const resolved = locale === 'auto' ? resolveSystemLanguage() : locale
  await loadPclLocale(resolved)
  if (change !== languageChange) return
  currentLocale.value = resolved
  languagePreference.value = locale
  if (typeof document !== 'undefined') document.documentElement.lang = resolved
  try {
    localStorage.setItem('pcl-language', locale)
  } catch (error) {
    console.warn('Unable to save language preference', error)
  }
}

export async function initializePclLocale(): Promise<void> {
  let locale = 'auto'
  try {
    const stored = localStorage.getItem('pcl-language') || 'auto'
    locale = stored.toLowerCase() === 'auto' ? 'auto' : normalizeLocale(stored)
    setPclFormatLocale(localStorage.getItem('pcl-format-locale') || 'auto')
  } catch (error) {
    console.warn('Unable to read language preference', error)
  }
  try {
    await setPclLocale(locale)
  } catch (error) {
    console.warn('Unable to load language; using zh-CN', error)
    currentLocale.value = 'zh-CN'
    languagePreference.value = locale
    if (typeof document !== 'undefined') document.documentElement.lang = 'zh-CN'
  }
}

if (typeof window !== 'undefined' && typeof window.addEventListener === 'function') {
  window.addEventListener('languagechange', () => {
    systemFormatLocale.value = Intl.DateTimeFormat().resolvedOptions().locale
    if (languagePreference.value === 'auto')
      void setPclLocale('auto').catch((error) =>
        console.warn('Unable to update system language', error),
      )
  })
}

export function pclText(value = '', params: readonly unknown[] = []): string {
  const strings = loaded[currentLocale.value] || pclStrings
  const translated =
    strings[value] ??
    pclStrings[value] ??
    value.replace(
      /\{(?:Dynamic|Static)Resource ([^}]+)\}/g,
      (all, key: string) => strings[key] ?? pclStrings[key] ?? all,
    )
  return translated
    .replace(/PCL CE/g, 'PCL.Proto')
    .replace(/PCL-CE 文件夹/g, 'PCL.Proto 文件夹')
    .replace(/\{(\d+)\}/g, (all, index: string) =>
      params[Number(index)] == null ? all : String(params[Number(index)]),
    )
}
