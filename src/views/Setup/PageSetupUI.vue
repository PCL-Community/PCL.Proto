<script setup lang="ts">
import { useRouter } from 'vue-router'
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import { appearance } from '@/api/appearance'
import { homepagePresets, homepageRevision } from '@/api/customHomepage'
import { isTauri, invoke } from '@tauri-apps/api/core'
import { computed, ref, toRef } from 'vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import { openExternal } from '@/util/externalLink'
import sideTip from '@/composables/sideTip'
withDefaults(defineProps<{ hasBackground?: boolean; hasMusic?: boolean }>(), {
  hasBackground: false,
  hasMusic: false,
})
const values = toRef(appearance, 'values')
function editHomepage() {
  void router.push('/homepageeditor')
}
const homepageOptions = [
  { key: '0', title: 'Setup.Ui.Homepage.Blank' },
  { key: '3', title: 'Setup.Ui.Homepage.Preset' },
  { key: '1', title: 'Setup.Ui.Homepage.LocalFile' },
  { key: '2', title: 'Setup.Ui.Homepage.NetUpdate' },
]
const homepagePresetOptions = computed(() =>
  homepagePresets.map((preset) => ({
    key: preset.key,
    text: pclText(
      preset.key === 'proto' ? 'Proto.About.Welcome' : `Setup.Ui.Homepage.Preset.${preset.name}`,
    ),
  })),
)
function setHomepageType(value: string) {
  values.value.UiCustomType = value
}
function refreshHomepage() {
  homepageRevision.value++
  sideTip.show(pclText('Setup.Ui.Homepage.Refresh.Success'), 'success')
}
const homepageWarningDismissed = ref(false)
try {
  homepageWarningDismissed.value = localStorage.getItem('pcl-homepage:warning-dismissed') === 'true'
} catch {
  /* Keep the warning visible when storage is unavailable. */
}
function dismissHomepageWarning() {
  homepageWarningDismissed.value = true
  try {
    localStorage.setItem('pcl-homepage:warning-dismissed', 'true')
  } catch {
    /* The dismissal remains active for this session. */
  }
}
const fonts = ref<string[]>([])
const fontOptions = computed(() => [
  { key: '', text: pclText('Common.Option.Default') },
  ...Array.from(
    new Set(
      [...fonts.value, values.value.ComboUiFont, values.value.ComboUiMotdFont].filter(
        (font) => font && font !== 'system-ui',
      ),
    ),
  )
    .sort()
    .map((font) => ({ key: font, text: font })),
])
if (isTauri())
  void invoke<string[]>('system_fonts')
    .then((result) => {
      fonts.value = result
    })
    .catch(console.error)

const router = useRouter()
const text = pclText
</script>
<template>
  <div class="pcl-page">
    <PCard :title="text('Setup.Ui.Basic.Title')">
      <div style="margin: 0px 25px 10px 25px; display: grid">
        <div
          style="
            grid-row: 1 / span 1;
            grid-column: 1 / span 1;
            display: flex;
            flex-direction: column;
          "
        >
          <div
            style="
              margin: 0px 0px 6px 0px;
              display: grid;
              grid-template-columns: auto minmax(0, 1fr) 30px auto minmax(0, 1fr);
              grid-template-rows: auto auto auto;
            "
          >
            <span
              style="
                margin: 0px 20px 0px 0px;
                grid-row: 1 / span 1;
                grid-column: 1 / span 1;
                justify-self: start;
                align-self: center;
              "
              class="pcl-text"
              >{{ text('Setup.Ui.Basic.Opacity') }}</span
            >
            <input
              disabled
              style="grid-row: 1 / span 1; grid-column: 2 / span 5"
              type="range"
              min="0"
              max="600"
              v-model.number="values['SliderLauncherOpacity']"
            />
          </div>
          <div
            style="
              height: 28px;
              margin: 0px 0px 7px 0px;
              display: grid;
              grid-template-columns:
                auto minmax(0, 1fr) 92px minmax(0, 1fr) 92px
                minmax(0, 1fr);
            "
          >
            <span
              style="
                grid-row: 1 / span 1;
                grid-column: 1 / span 1;
                margin-right: 20px;
                justify-self: start;
                align-self: center;
              "
              class="pcl-text"
              >{{ text('Setup.Ui.Theme.Title') }}</span
            >
            <PComboBox
              style="grid-row: 1 / span 1; grid-column: 2 / span 1"
              :options="[
                { key: '0', text: text('Setup.Ui.Theme.Light') },
                { key: '1', text: text('Setup.Ui.Theme.Dark') },
                { key: '2', text: text('Setup.Ui.Theme.FollowSystem') },
              ]"
              v-model="values.ComboDarkMode"
            />
            <span
              style="
                grid-row: 1 / span 1;
                grid-column: 3 / span 1;
                justify-self: center;
                align-self: center;
              "
              class="pcl-text"
              >{{ text('Setup.Ui.Theme.LightColor') }}</span
            >
            <PComboBox
              style="grid-row: 1 / span 1; grid-column: 4 / span 1"
              :options="
                ['SkyBlue', 'CatBlue', 'CrashBlue'].map((color, index) => ({
                  key: String(index),
                  text: text(`Setup.Ui.Theme.Color.${color}`),
                }))
              "
              v-model="values.ComboLightColor"
            />
            <span
              style="
                grid-row: 1 / span 1;
                grid-column: 5 / span 1;
                justify-self: center;
                align-self: center;
              "
              class="pcl-text"
              >{{ text('Setup.Ui.Theme.DarkColor') }}</span
            >
            <PComboBox
              style="grid-row: 1 / span 1; grid-column: 6 / span 1"
              :options="
                ['SkyBlue', 'CatBlue', 'CrashBlue'].map((color, index) => ({
                  key: String(index),
                  text: text(`Setup.Ui.Theme.Color.${color}`),
                }))
              "
              v-model="values.ComboDarkColor"
            />
          </div>
          <PCheckBox
            disabled
            style="height: 22px; margin: 0px 0px 7px -1px"
            id="PageSetupUI-CheckLauncherLogo"
            :label="text('Setup.Ui.Basic.ShowLogo')"
            v-model="values.CheckLauncherLogo"
          />
          <PCheckBox
            disabled
            style="height: 22px; margin: 0px 0px 7px -1px"
            id="PageSetupUI-CheckLockWindowSize"
            :label="text('Setup.Ui.Basic.LockWindow')"
            v-model="values.CheckLockWindowSize"
          />
          <PCheckBox
            disabled
            style="height: 22px; margin: 0px 0px 7px -1px"
            id="PageSetupUI-CheckShowLaunchingHint"
            :label="text('Setup.Ui.Basic.ShowTriviaHint')"
            v-model="values.CheckShowLaunchingHint"
          />
          <div style="margin: 0px 0px 10px 0px; display: flex; flex-direction: column">
            <PCheckBox
              disabled
              style="height: 22px"
              :title="text('Setup.Ui.Basic.AdvancedMaterial.ToolTip')"
              id="PageSetupUI-CheckBlur"
              :label="text('Setup.Ui.Basic.AdvancedMaterial')"
              v-model="values.CheckBlur"
            />
            <div
              v-if="values.CheckBlur"
              style="
                margin: 10px 0px 0px 0px;
                display: grid;
                grid-template-columns: auto auto auto minmax(0, 1fr);
                grid-template-rows: minmax(0, 1fr) 10px minmax(0, 1fr);
              "
            >
              <span
                style="
                  grid-row: 1 / span 1;
                  grid-column: 1 / span 1;
                  justify-self: start;
                  align-self: center;
                "
                class="pcl-text"
                >{{ text('Setup.Ui.Basic.BlurRadius') }}</span
              >
              <input
                disabled
                style="grid-row: 1 / span 1; grid-column: 2 / span 3"
                :title="text('Setup.Ui.Basic.BlurRadius.ToolTip')"
                type="range"
                min="0"
                max="40"
                v-model.number="values['SliderBlurValue']"
              />
              <span
                style="
                  grid-row: 3 / span 1;
                  grid-column: 3 / span 1;
                  justify-self: center;
                  align-self: center;
                "
                class="pcl-text"
                >{{ text('Setup.Ui.Basic.BlurSamplingRate') }}</span
              >
              <input
                disabled
                style="grid-row: 3 / span 1; grid-column: 4 / span 1"
                :title="text('Setup.Ui.Basic.BlurSamplingRate.ToolTip')"
                type="range"
                min="0"
                max="100"
                v-model.number="values['SliderBlurSamplingRate']"
              />
              <span
                style="
                  grid-row: 3 / span 1;
                  grid-column: 1 / span 1;
                  justify-self: start;
                  align-self: center;
                "
                class="pcl-text"
                >{{ text('Setup.Ui.Basic.BlurMethod') }}</span
              >
              <PComboBox
                disabled
                style="
                  height: 28px;
                  min-width: 100px;
                  grid-row: 3 / span 1;
                  grid-column: 2 / span 1;
                "
                :title="text('Setup.Ui.Basic.BlurMethod.ToolTip')"
                :options="[
                  {
                    key: '0',
                    text: text('Setup.Ui.Basic.BlurMethod.Gaussian'),
                    title: text('Setup.Ui.Basic.BlurMethod.Gaussian.ToolTip'),
                  },
                  {
                    key: '1',
                    text: text('Setup.Ui.Basic.BlurMethod.Box'),
                    title: text('Setup.Ui.Basic.BlurMethod.Box.ToolTip'),
                  },
                ]"
                v-model="values.ComboBlurType"
              />
            </div>
          </div>
        </div>
      </div>
    </PCard>
    <PCard :title="text('Setup.Ui.Font.Title')">
      <div
        style="
          margin: 0px 25px 15px 25px;
          display: grid;
          grid-template-columns: auto 25px minmax(0, 1fr);
          grid-template-rows: 30px 10px minmax(0, 1fr);
        "
      >
        <span
          style="grid-row: 1 / span 1; grid-column: 1 / span 1; align-self: center"
          class="pcl-text"
          >{{ text('Setup.Ui.Font.Global') }}</span
        >
        <PComboBox
          style="grid-row: 1 / span 1; grid-column: 3 / span 1"
          :options="fontOptions"
          :title="text('Setup.Ui.Font.Global.ToolTip')"
          v-model="values.ComboUiFont"
        />
        <span
          style="grid-row: 3 / span 1; grid-column: 1 / span 1; align-self: center"
          class="pcl-text"
          >{{ text('Setup.Ui.Font.Motd') }}</span
        >
        <PComboBox
          style="grid-row: 3 / span 1; grid-column: 3 / span 1"
          :options="fontOptions"
          :title="text('Setup.Ui.Font.Motd.ToolTip')"
          v-model="values.ComboUiMotdFont"
        />
      </div>
    </PCard>
    <PCard :title="text('Setup.Ui.Background.TitleDefault')">
      <div style="margin: 0px 25px 20px 25px; display: flex; flex-direction: column">
        <template v-if="hasBackground">
          <div class="pcl-form-row">
            <label>{{ text('Setup.Ui.Background.ContentAdapt') }}</label
            ><PComboBox
              disabled
              :options="
                [
                  'Smart',
                  'Center',
                  'Fit',
                  'Stretch',
                  'Tile',
                  'TopLeft',
                  'TopRight',
                  'BottomLeft',
                  'BottomRight',
                ].map((key, index) => ({
                  key: String(index),
                  text: text(`Setup.Ui.Background.Suit.${key}`),
                }))
              "
              :model-value="String(values.ComboBackgroundSuit)"
              @update:model-value="(value) => (values.ComboBackgroundSuit = value || '0')"
            />
          </div>
          <div class="pcl-form-row">
            <label for="background-opacity">{{ text('Setup.Ui.Basic.Opacity2') }}</label
            ><input
              disabled
              id="background-opacity"
              type="range"
              min="0"
              max="1000"
              v-model.number="values.SliderBackgroundOpacity"
            />
          </div>
          <div class="pcl-form-row">
            <label for="background-blur">{{ text('Setup.Ui.Background.Blur') }}</label
            ><input
              disabled
              id="background-blur"
              type="range"
              min="0"
              max="40"
              v-model.number="values.SliderBackgroundBlur"
            />
          </div>
        </template>
        <PCheckBox
          disabled
          style="height: 22px; margin: 0px 0px 8px -1px"
          :title="text('Setup.Ui.Background.PauseVideo.ToolTip')"
          id="PageSetupUI-CheckAutoPauseVideo"
          :label="text('Setup.Ui.Background.PauseVideo')"
          v-model="values.CheckAutoPauseVideo"
        />
        <PCheckBox
          style="height: 22px; margin: 0px 0px 8px -1px"
          id="PageSetupUI-CheckBackgroundColorful"
          :label="text('Setup.Ui.Background.ColorOverlay')"
          v-model="values.CheckBackgroundColorful"
        />
        <div
          style="
            height: 35px;
            margin: 2px 0px 0px 0px;
            display: grid;
            grid-template-columns: auto auto auto;
          "
        >
          <PButton
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
            "
            :title="text('Setup.Ui.Background.OpenFolder.ToolTip')"
            @click="showUnavailable"
            >{{ text('Common.Action.OpenFolder') }}</PButton
          >
          <PButton
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 2 / span 1;
            "
            :title="text('Setup.Ui.Background.Refresh.ToolTip')"
            @click="showUnavailable"
            >{{ text('Setup.Ui.Background.Refresh') }}</PButton
          ><PButton v-if="hasBackground" type="warn" @click="showUnavailable">{{
            text('Setup.Ui.Background.Clear')
          }}</PButton>
        </div>
      </div>
    </PCard>
    <PCard :title="text('Setup.Ui.Music.Title')">
      <div style="margin: 2px 25px 20px 25px; display: flex; flex-direction: column">
        <template v-if="hasMusic">
          <div class="pcl-form-row">
            <label for="music-volume">{{ text('Setup.Ui.Music.Volume') }}</label
            ><input
              disabled
              id="music-volume"
              type="range"
              min="0"
              max="1000"
              v-model.number="values.SliderMusicVolume"
            />
          </div>
          <div style="display: flex; flex-direction: column; gap: 5px; margin-bottom: 8px">
            <PCheckBox
              disabled
              v-for="setting in [
                { id: 'CheckMusicRandom', key: 'Shuffle' },
                { id: 'CheckMusicAuto', key: 'AutoStart' },
                { id: 'CheckMusicStart', key: 'StartInGame' },
                { id: 'CheckMusicStop', key: 'StopInGame' },
                { id: 'CheckMusicSMTC', key: 'Smtc' },
              ]"
              :key="setting.id"
              :id="`PageSetupUI-${setting.id}`"
              :label="text(`Setup.Ui.Music.${setting.key}`)"
              :model-value="Boolean(values[setting.id])"
              @update:model-value="(value) => (values[setting.id] = value)"
            />
          </div>
        </template>
        <div style="height: 35px; display: grid; grid-template-columns: auto auto auto">
          <PButton
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
            "
            :title="text('Setup.Ui.Music.OpenFolder.ToolTip')"
            @click="showUnavailable"
            >{{ text('Common.Action.OpenFolder') }}</PButton
          >
          <PButton
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 2 / span 1;
            "
            :title="text('Setup.Ui.Music.Refresh.ToolTip')"
            @click="showUnavailable"
            >{{ text('Setup.Ui.Music.Refresh') }}</PButton
          ><PButton v-if="hasMusic" type="warn" @click="showUnavailable">{{
            text('Setup.Ui.Music.Clear')
          }}</PButton>
        </div>
      </div>
    </PCard>
    <PCard :title="text('Setup.Ui.Logo.Title')">
      <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
        <div class="setup-radio-options">
          <label class="pcl-radio"
            ><input disabled type="radio" v-model="values['UiLogoType']" value="0" />{{
              text('Setup.Ui.Logo.None')
            }}</label
          >
          <label class="pcl-radio"
            ><input disabled type="radio" v-model="values['UiLogoType']" value="1" />{{
              text('Common.Option.Default')
            }}</label
          >
          <label class="pcl-radio"
            ><input disabled type="radio" v-model="values['UiLogoType']" value="2" />{{
              text('Setup.Ui.Logo.Text')
            }}</label
          >
          <label class="pcl-radio"
            ><input disabled type="radio" v-model="values['UiLogoType']" value="3" />{{
              text('Setup.Ui.Logo.Image')
            }}</label
          >
        </div>
        <PCheckBox
          disabled
          v-if="values.UiLogoType === '0'"
          style="margin: 8px 0 3px"
          id="PageSetupUI-CheckLogoLeft"
          :label="text('Setup.Ui.Logo.LeftAlign')"
          v-model="values.CheckLogoLeft"
        />
        <div
          style="
            margin: 14px 0px 5px 0px;
            display: grid;
            grid-template-columns: auto minmax(0, 1fr);
          "
          v-if="values.UiLogoType === '2'"
        >
          <span
            style="
              margin: 0px 25px 0px 0px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
              justify-self: start;
              align-self: center;
            "
            class="pcl-text"
            >{{ text('Setup.Ui.Logo.TextLabel') }}</span
          >
          <PTextBox
            disabled
            style="grid-row: 1 / span 1; grid-column: 2 / span 1"
            :placeholder="text('')"
            v-model="values.TextLogoText"
          />
        </div>
        <div
          class="logo-image-actions"
          style="
            height: 35px;
            margin: 14px 0px 5px 0px;
            display: grid;
            grid-template-columns: auto auto;
          "
          v-if="values.UiLogoType === '3'"
        >
          <PButton
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
            "
            @click="showUnavailable"
            >{{ text('Setup.Ui.Logo.ChangeImage') }}</PButton
          >
          <PButton
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 2 / span 1;
            "
            @click="showUnavailable"
            type="warn"
            >{{ text('Setup.Ui.Logo.ClearImage') }}</PButton
          >
        </div>
      </div>
    </PCard>
    <PCard :title="text('Setup.Ui.Homepage.Title')">
      <div class="homepage-settings">
        <div class="setup-radio-options homepage-options">
          <label v-for="option in homepageOptions" :key="option.key" class="pcl-radio">
            <input
              type="radio"
              name="homepage-type"
              :checked="values.UiCustomType === option.key"
              :value="option.key"
              @change="setHomepageType(option.key)"
            />{{ text(option.title) }}
          </label>
        </div>
        <PHint v-if="values.UiCustomType === '1'" severity="info" class="homepage-hint">{{
          text('Proto.Homepage.LocalHint')
        }}</PHint>
        <PHint v-else-if="values.UiCustomType === '2'" severity="info" class="homepage-hint">
          {{ text('Setup.Ui.Homepage.NetUpdate.Hint') }}
          <button
            type="button"
            class="homepage-hint-link"
            @click="openExternal('https://github.com/PCL-Community/PCL-CE/discussions')"
          >
            {{ text('Proto.Homepage.Submit') }}
          </button>
        </PHint>
        <PHint
          v-if="
            (values.UiCustomType === '1' || values.UiCustomType === '2') &&
            !homepageWarningDismissed
          "
          severity="warning"
          closable
          @close="dismissHomepageWarning"
          >{{ text('Setup.Ui.Homepage.NetWarning') }}</PHint
        >
        <div v-if="values.UiCustomType === '2'" class="pcl-form-row">
          <label for="homepage-url">{{ text('Setup.Ui.Homepage.DownloadUrl') }}</label>
          <PTextBox
            input-id="homepage-url"
            v-model="values.TextCustomNet"
            placeholder="https://example.com/Custom.xaml"
          />
        </div>
        <div v-if="values.UiCustomType === '3'" class="pcl-form-row">
          <label>{{ text('Setup.Ui.Homepage.PresetLabel') }}</label>
          <PComboBox
            v-model="values.ComboCustomPreset"
            :options="homepagePresetOptions"
            :aria-label="text('Setup.Ui.Homepage.PresetLabel')"
          />
        </div>
        <div v-if="values.UiCustomType === '1'" class="homepage-actions">
          <PButton type="tint" @click="refreshHomepage">{{
            text('Setup.Ui.Homepage.Refresh')
          }}</PButton>
          <PButton @click="editHomepage">{{ text('Proto.Title.HomepageEditor') }}</PButton>
          <PButton @click="openExternal('https://docs.pclc.cc/ce/customization/xaml-format')">{{
            text('Setup.Ui.Homepage.Tutorial')
          }}</PButton>
        </div>
      </div>
    </PCard>
    <PCard :title="text('Setup.Ui.FeatureHide.Title')">
      <div style="margin: 0px 15px 15px 25px; display: flex; flex-direction: column">
        <span style="margin: 1px 0px 1px 0px" class="pcl-text">{{
          text('Setup.Ui.FeatureHide.Description')
        }}</span>
        <div
          style="
            display: grid;
            grid-template-columns:
              auto 18px minmax(0, 0.8fr) minmax(0, 0.9fr) minmax(0, 0.8fr)
              minmax(0, 0.8fr) minmax(0, 1fr) minmax(0, 1fr);
            grid-template-rows: 4px 30px 30px 30px 30px 30px 30px 30px auto;
          "
        >
          <span
            style="
              grid-row: 2 / span 1;
              grid-column: 1 / span 1;
              justify-self: start;
              align-self: center;
            "
            class="pcl-text"
            >{{ text('Setup.Ui.FeatureHide.MainPage') }}</span
          >
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 2 / span 1; grid-column: 3 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenPageDownload"
            :label="text('Setup.Ui.FeatureHide.Item.Download')"
            v-model="values.CheckHiddenPageDownload"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 2 / span 1; grid-column: 4 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenPageSetup"
            :label="text('Setup.Ui.FeatureHide.Item.Settings')"
            v-model="values.CheckHiddenPageSetup"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 2 / span 1; grid-column: 5 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenPageTools"
            :label="text('Setup.Ui.FeatureHide.Item.Tools')"
            v-model="values.CheckHiddenPageTools"
          />
          <span
            style="
              grid-row: 3 / span 1;
              grid-column: 1 / span 1;
              justify-self: start;
              align-self: center;
            "
            class="pcl-text"
            >{{ text('Setup.Ui.FeatureHide.SubSetup') }}</span
          >
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 3 / span 1; grid-column: 3 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenSetupLaunch"
            :label="text('Setup.Ui.FeatureHide.Item.Launch')"
            v-model="values.CheckHiddenSetupLaunch"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 3 / span 1; grid-column: 4 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenSetupJava"
            :label="text('Java')"
            v-model="values.CheckHiddenSetupJava"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 3 / span 1; grid-column: 5 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenSetupGameManage"
            :label="text('Setup.Ui.FeatureHide.Item.GameManage')"
            v-model="values.CheckHiddenSetupGameManage"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 3 / span 1; grid-column: 6 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenSetupGameLink"
            :label="text('Setup.Ui.FeatureHide.Item.GameLink')"
            v-model="values.CheckHiddenSetupGameLink"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 3 / span 1; grid-column: 7 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenSetupUI"
            :label="text('Setup.Ui.FeatureHide.Item.Ui')"
            v-model="values.CheckHiddenSetupUI"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 4 / span 1; grid-column: 3 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenSetupLauncherLanguage"
            :label="text('Setup.Ui.FeatureHide.Item.Language')"
            v-model="values.CheckHiddenSetupLauncherLanguage"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 4 / span 1; grid-column: 4 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenLauncherMisc"
            :label="text('Setup.Ui.FeatureHide.Item.Misc')"
            v-model="values.CheckHiddenLauncherMisc"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 4 / span 1; grid-column: 5 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenSetupUpdate"
            :label="text('Setup.Ui.FeatureHide.Item.Update')"
            v-model="values.CheckHiddenSetupUpdate"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 4 / span 1; grid-column: 6 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenSetupAbout"
            :label="text('Setup.Ui.FeatureHide.Item.About')"
            v-model="values.CheckHiddenSetupAbout"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 4 / span 1; grid-column: 7 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenSetupFeedback"
            :label="text('Setup.Ui.FeatureHide.Item.Feedback')"
            v-model="values.CheckHiddenSetupFeedback"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 4 / span 1; grid-column: 8 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenSetupLog"
            :label="text('Setup.Ui.FeatureHide.Item.Log')"
            v-model="values.CheckHiddenSetupLog"
          />
          <span
            style="
              grid-row: 5 / span 1;
              grid-column: 1 / span 1;
              justify-self: start;
              align-self: center;
            "
            class="pcl-text"
            >{{ text('Setup.Ui.FeatureHide.SubTools') }}</span
          >
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 5 / span 1; grid-column: 3 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenToolsGameLink"
            :label="text('Setup.Ui.FeatureHide.Item.Lobby')"
            v-model="values.CheckHiddenToolsGameLink"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 5 / span 1; grid-column: 4 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenToolsTest"
            :label="text('Setup.Ui.FeatureHide.Item.Toolbox')"
            v-model="values.CheckHiddenToolsTest"
          />
          <span
            style="
              grid-row: 6 / span 1;
              grid-column: 1 / span 1;
              justify-self: start;
              align-self: center;
            "
            class="pcl-text"
            >{{ text('Setup.Ui.FeatureHide.SubInstance') }}</span
          >
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 6 / span 1; grid-column: 3 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenVersionEdit"
            :label="text('Setup.Ui.FeatureHide.Item.Edit')"
            v-model="values.CheckHiddenVersionEdit"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 6 / span 1; grid-column: 4 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenVersionExport"
            :label="text('Setup.Ui.FeatureHide.Item.Export')"
            v-model="values.CheckHiddenVersionExport"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 6 / span 1; grid-column: 5 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenVersionSave"
            :label="text('Setup.Ui.FeatureHide.Item.Save')"
            v-model="values.CheckHiddenVersionSave"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 6 / span 1; grid-column: 6 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenVersionScreenshot"
            :label="text('Setup.Ui.FeatureHide.Item.Screenshot')"
            v-model="values.CheckHiddenVersionScreenshot"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 6 / span 1; grid-column: 7 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenVersionMod"
            :label="text('Mod')"
            v-model="values.CheckHiddenVersionMod"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 7 / span 1; grid-column: 3 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenVersionResourcePack"
            :label="text('Setup.Ui.FeatureHide.Item.ResourcePack')"
            v-model="values.CheckHiddenVersionResourcePack"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 7 / span 1; grid-column: 4 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenVersionShader"
            :label="text('Setup.Ui.FeatureHide.Item.Shader')"
            v-model="values.CheckHiddenVersionShader"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 7 / span 1; grid-column: 5 / span 2; align-self: center"
            id="PageSetupUI-CheckHiddenVersionSchematic"
            :label="text('Setup.Ui.FeatureHide.Item.Schematic')"
            v-model="values.CheckHiddenVersionSchematic"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 7 / span 1; grid-column: 7 / span 1; align-self: center"
            id="PageSetupUI-CheckHiddenVersionServer"
            :label="text('Setup.Ui.FeatureHide.Item.Server')"
            v-model="values.CheckHiddenVersionServer"
          />
          <span
            style="
              grid-row: 8 / span 1;
              grid-column: 1 / span 1;
              justify-self: start;
              align-self: center;
            "
            class="pcl-text"
            >{{ text('Setup.Ui.FeatureHide.SpecificFeature') }}</span
          >
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 8 / span 1; grid-column: 3 / span 1; align-self: center"
            :title="text('Setup.Ui.FeatureHide.Item.InstanceSelect.ToolTip')"
            id="PageSetupUI-CheckHiddenFunctionSelect"
            :label="text('Setup.Ui.FeatureHide.Item.InstanceSelect')"
            v-model="values.CheckHiddenFunctionSelect"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 8 / span 1; grid-column: 4 / span 1; align-self: center"
            :title="text('Setup.Ui.FeatureHide.Item.ModUpdate.ToolTip')"
            id="PageSetupUI-CheckHiddenFunctionModUpdate"
            :label="text('Setup.Ui.FeatureHide.Item.ModUpdate')"
            v-model="values.CheckHiddenFunctionModUpdate"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 8 / span 1; grid-column: 5 / span 1; align-self: center"
            :title="text('Setup.Ui.FeatureHide.Item.FeatureHide.ToolTip')"
            id="PageSetupUI-CheckHiddenFunctionHidden"
            :label="text('Setup.Ui.FeatureHide.Item.FeatureHide')"
            v-model="values.CheckHiddenFunctionHidden"
          />
        </div>
      </div>
    </PCard>
  </div>
</template>

<style scoped>
.logo-image-actions {
  justify-content: start;
}
</style>

<style scoped>
.homepage-settings {
  display: flex;
  container-type: inline-size;
  flex-direction: column;
  gap: 15px;
  padding: 0 25px 20px;
}
.setup-radio-options {
  display: flex;
  margin-left: -1px;
}
.setup-radio-options > .pcl-radio {
  flex: 1 1 0;
  min-width: 0;
  min-height: 22px;
}
.homepage-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
}
.homepage-actions :deep(.mybutton) {
  min-height: 35px;
  min-width: 140px;
  padding: 0 13px;
}
.homepage-hint {
  white-space: pre-line;
}
.homepage-hint-link {
  border: 0;
  background: none;
  color: inherit;
  padding: 0 0 0 6px;
  cursor: pointer;
  text-decoration: underline;
  font: inherit;
}
</style>
