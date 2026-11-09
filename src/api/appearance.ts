import { watch } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { usePageSettings } from '@/composables/usePageSettings'
import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'

export const appearance = usePageSettings('PageSetupUI', {
  ComboUiFont: '',
  ComboUiMotdFont: '',
  ComboDarkMode: '2',
  ComboLightColor: '1',
  ComboDarkColor: '1',
  CheckBackgroundColorful: true,
  UiLogoType: '1',
  CheckLogoLeft: false,
  TextLogoText: '',
})

for (const key of ['ComboUiFont', 'ComboUiMotdFont'] as const) {
  if (appearance.values[key] === 'system-ui') appearance.values[key] = ''
}

const systemDark = window.matchMedia('(prefers-color-scheme: dark)')
// Precomputed with CE's Unicolour 8.0.0 OKLCH gamut mapping, in theme order.
const themes = [
  [
    '#0A4662 #00A5F0 #00B5FF #1DC4FF #92E0FF #CDF5FF #D6F8FF #DFFCFF #81D9F5 #E1F5FF #9CD0FF',
    '#323B47 #1662B6 #2078DA #5291DF #9EC1EE #D8E6F8 #E1ECFA #EBF2FC #92D3FF #E6F3FF #B3C8FF',
    '#353A47 #3450BA #4263DE #5E7FE0 #9AB0E7 #CCD5EC #D4DCED #DDE2ED #9FCFFF #EAF2FF #C1C3FF',
  ],
  [
    '#D5F8FF #76D7FF #00BAFF #1DC4FF #009BE2 #0072AB #006EA4 #00699D #00131E #07161F #000C32',
    '#E7F3FF #68B0FF #2A80E3 #5291DF #38577E #18222F #151C26 #12161D #001028 #0B1521 #040536',
    '#E9F0FF #759BFF #486AE6 #5E7FE0 #3B4D7D #181E2E #141925 #11141B #000A35 #0F1421 #0E0235',
  ],
] as const

function applyAppearance() {
  const v = appearance.values
  const dark = v.ComboDarkMode === '1' || (v.ComboDarkMode === '2' && systemDark.matches)
  const index = Number(dark ? v.ComboDarkColor : v.ComboLightColor)
  const palette = themes[dark ? 1 : 0]
  const colors = (palette[index] || palette[1]).split(' ')
  const root = document.documentElement
  root.classList.toggle('dark', dark)
  root.style.setProperty('--hint-hue-offset', String([-20, 0, 13][index] ?? 0))
  colors.slice(0, 8).forEach((color, i) => root.style.setProperty(`--ce-color-${i + 1}`, color))
  root.style.setProperty(
    '--base-background',
    v.CheckBackgroundColorful
      ? `linear-gradient(to bottom left, ${colors[8]} -10%, ${colors[9]} 40%, ${colors[10]} 110%)`
      : 'var(--color-background)',
  )
  const font = (name: unknown) =>
    typeof name === 'string' && name && name !== 'system-ui' ? `${JSON.stringify(name)}, ` : ''
  root.style.setProperty('--custom-font', `${font(v.ComboUiFont)}var(--default-font)`)
  root.style.setProperty('--motd-font', `${font(v.ComboUiMotdFont)}var(--default-font)`)
}

watch(appearance.values, applyAppearance, { deep: true, flush: 'sync', immediate: true })
systemDark.addEventListener('change', applyAppearance)

if (isTauri()) {
  const window = getCurrentWindow()
  let pending = Promise.resolve()
  watch(
    () => appearance.values.CheckLockWindowSize,
    (locked) => {
      pending = pending
        .then(() => window.setResizable(!locked))
        .catch((error) => {
          console.error('Failed to update window resizability:', error)
          sideTip.show(pclText('Setup.Ui.Error.OperationFailed'), 'warn')
        })
    },
    { immediate: true },
  )
}
