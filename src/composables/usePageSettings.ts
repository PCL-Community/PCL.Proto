import sourceDefaults from '@/assets/setting-defaults.json'
import { computed, reactive, watch } from 'vue'
import { pclText } from '@/api/pclLocale'
import { useSelectedInstance } from '@/stores/gameLaunch'
import sideTip from '@/composables/sideTip'

type SettingValue = string | boolean | number
type PageDefaults<Page extends string> = Page extends keyof typeof sourceDefaults
  ? (typeof sourceDefaults)[Page]
  : Record<never, never>
type Settings<T> = {
  [Key in keyof T]: T[Key] extends boolean ? boolean : T[Key] extends number ? number : string
} & Record<string, SettingValue>

export function usePageSettings<
  Page extends string,
  Defaults extends Record<string, SettingValue> = Record<never, never>,
>(page: Page, defaults: Defaults = {} as Defaults, excludedKeys: string[] = []) {
  const pageDefaults = (sourceDefaults as Record<string, Record<string, SettingValue>>)[page] || {}
  const initialDefaults = {
    ...Object.fromEntries(
      Object.entries(pageDefaults).filter(
        ([key]) =>
          !['ComboArgumentJava', 'CheckAdvanceJava', 'ComboUiFont', 'ComboUiMotdFont'].includes(
            key,
          ),
      ),
    ),
    ...defaults,
  }
  const instance = page.startsWith('PageInstance') ? useSelectedInstance() : null
  const storageKey = computed(() =>
    instance && !instance.instance_info?.directory
      ? null
      : instance
        ? `pcl-instance-ui:${page}:${encodeURIComponent(instance.instance_info!.directory)}`
        : `pcl-ui:${page}:global`,
  )
  function loadValues() {
    if (!storageKey.value) return {}
    try {
      const parsed: unknown = JSON.parse(localStorage.getItem(storageKey.value) || '{}')
      if (parsed && typeof parsed === 'object' && !Array.isArray(parsed))
        return Object.fromEntries(
          Object.entries(parsed).filter(
            ([key, value]) =>
              !/password|token|secret|cdk/i.test(key) &&
              !excludedKeys.includes(key) &&
              !['ComboArgumentJava', 'CheckAdvanceJava'].includes(key) &&
              ['string', 'number', 'boolean'].includes(typeof value) &&
              (!(key in initialDefaults) || typeof value === typeof initialDefaults[String(key)]),
          ),
        )
    } catch {
      // Invalid saved preferences must not prevent the page from opening.
    }
    return {}
  }
  const values = reactive({ ...initialDefaults, ...loadValues() }) as Settings<
    PageDefaults<Page> & Defaults
  >
  let loading = false
  watch(
    values,
    () => {
      if (loading || !storageKey.value) return
      try {
        localStorage.setItem(
          storageKey.value,
          JSON.stringify(
            Object.fromEntries(
              Object.entries(values).filter(
                ([key]) => !excludedKeys.includes(key) && !/password|token|secret|cdk/i.test(key),
              ),
            ),
          ),
        )
      } catch {
        sideTip.show(pclText('Proto.Storage.SettingsFailed'), 'warn')
      }
    },
    { deep: true, flush: 'sync' },
  )
  watch(
    storageKey,
    () => {
      loading = true
      try {
        Object.keys(values).forEach((key) => delete values[key])
        Object.assign(values, initialDefaults, loadValues())
      } finally {
        loading = false
      }
    },
    { flush: 'sync' },
  )
  function reset(key?: keyof typeof values) {
    if (key) {
      Object.assign(values, { [key]: initialDefaults[String(key)] })
    } else {
      Object.keys(values).forEach((key) => delete values[key])
      Object.assign(values, initialDefaults)
    }
  }
  return { values, reset }
}
