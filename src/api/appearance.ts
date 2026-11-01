import { watch } from 'vue'
import { usePageSettings } from '@/composables/usePageSettings'

export const appearance = usePageSettings('PageSetupUI', {
  ComboUiFont: '',
  ComboUiMotdFont: '',
  ComboDarkMode: '2',
  ComboLightColor: '1',
  ComboDarkColor: '1',
  CheckBackgroundColorful: true,
  UiLogoType: '0',
})

for (const key of ['ComboUiFont', 'ComboUiMotdFont'] as const) {
  if (appearance.values[key] === 'system-ui') appearance.values[key] = ''
}

const systemDark = window.matchMedia('(prefers-color-scheme: dark)')
const themes = [
  [235, 0.36, 0.2],
  [255, 0, -0.2],
  [268, -0.05, -0.1],
] as const
const chroma = [0.025, 0.188, 0.213, 0.168, 0.093, 0.036, 0.028, 0.018]
const adjust = (v: number, a: number) => (a > 0 ? v + (1 - v) * a : v * (1 + a))

function applyAppearance() {
  const v = appearance.values
  const dark = v.ComboDarkMode === '1' || (v.ComboDarkMode === '2' && systemDark.matches)
  const index = Number(dark ? v.ComboDarkColor : v.ComboLightColor)
  const [hue, light, saturation] = themes[index] || themes[1]
  const levels = dark
    ? [0.96, 0.75, 0.6, 0.65, 0.45, 0.25, 0.225, 0.2]
    : [0.35, 0.5, 0.575, 0.65, 0.8, 0.92, 0.94, 0.96]
  const root = document.documentElement
  root.classList.toggle('dark', dark)
  levels.forEach((level, i) =>
    root.style.setProperty(
      `--ce-color-${i + 1}`,
      `oklch(${adjust(level, light * (i === 0 ? 0.1 : 1))} ${adjust(chroma[i]!, saturation * (i === 0 ? 0.25 : 1))} ${hue})`,
    ),
  )
  const backgroundLight = dark ? 0.2 : 1
  root.style.setProperty(
    '--base-background',
    v.CheckBackgroundColorful
      ? `linear-gradient(to bottom left, oklch(${0.84 * backgroundLight} 0.093 ${hue - 15}) -10%, oklch(${0.96 * backgroundLight} 0.028 ${hue}) 40%, oklch(${0.84 * backgroundLight} 0.093 ${hue + 15}) 110%)`
      : 'var(--color-background)',
  )
  const font = (name: unknown) =>
    typeof name === 'string' && name && name !== 'system-ui' ? `${JSON.stringify(name)}, ` : ''
  root.style.setProperty('--custom-font', `${font(v.ComboUiFont)}var(--default-font)`)
  root.style.setProperty('--motd-font', `${font(v.ComboUiMotdFont)}var(--default-font)`)
}

watch(appearance.values, applyAppearance, { deep: true, flush: 'sync', immediate: true })
systemDark.addEventListener('change', applyAppearance)
