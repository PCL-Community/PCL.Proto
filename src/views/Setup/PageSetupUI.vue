<script setup lang="ts">
import { useRouter } from 'vue-router'
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import { appearance } from '@/api/appearance'
import { isTauri, invoke } from '@tauri-apps/api/core'
import { computed, ref } from 'vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import { openExternal } from '@/util/externalLink'
withDefaults(defineProps<{ hasBackground?: boolean; hasMusic?: boolean }>(), {
  hasBackground: false,
  hasMusic: false,
})
const { values } = appearance
const fonts = ref<string[]>([])
const fontOptions = computed(() => [
  { key: '', text: pclText('Common.Option.Default') },
  ...Array.from(
    new Set(
      [...fonts.value, values.ComboUiFont, values.ComboUiMotdFont].filter(
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
    <p class="settings-unavailable">{{ text('Proto.Settings.Unavailable') }}</p>
    <div>
      <div data-control="PanBack">
        <div
          data-control="PanMain"
          style="margin: 10px 25px; display: flex; flex-direction: column"
        >
          <PCard
            data-control="CardLauncher"
            style="margin: 15px 0px"
            :title="text('Setup.Ui.Basic.Title')"
          >
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
                    data-control="SliderLauncherOpacity"
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
                    data-control="ComboDarkMode"
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
                    data-control="ComboLightColor"
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
                    data-control="ComboDarkColor"
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
                  data-control="CheckLauncherLogo"
                  style="height: 22px; margin: 0px 0px 7px -1px"
                  id="PageSetupUI-CheckLauncherLogo"
                  :label="text('Setup.Ui.Basic.ShowLogo')"
                  v-model="values.CheckLauncherLogo"
                />
                <PCheckBox
                  disabled
                  data-control="CheckLockWindowSize"
                  style="height: 22px; margin: 0px 0px 7px -1px"
                  id="PageSetupUI-CheckLockWindowSize"
                  :label="text('Setup.Ui.Basic.LockWindow')"
                  v-model="values.CheckLockWindowSize"
                />
                <PCheckBox
                  disabled
                  data-control="CheckShowLaunchingHint"
                  style="height: 22px; margin: 0px 0px 7px -1px"
                  id="PageSetupUI-CheckShowLaunchingHint"
                  :label="text('Setup.Ui.Basic.ShowTriviaHint')"
                  v-model="values.CheckShowLaunchingHint"
                />
                <div
                  data-control="PanelBlur"
                  style="margin: 0px 0px 10px 0px; display: flex; flex-direction: column"
                >
                  <PCheckBox
                    disabled
                    data-control="CheckBlur"
                    style="height: 22px"
                    :title="text('Setup.Ui.Basic.AdvancedMaterial.ToolTip')"
                    id="PageSetupUI-CheckBlur"
                    :label="text('Setup.Ui.Basic.AdvancedMaterial')"
                    v-model="values.CheckBlur"
                  />
                  <div
                    v-if="values.CheckBlur"
                    data-control="PanBlurValue"
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
                      data-control="SliderBlurValue"
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
                      data-control="SliderBlurSamplingRate"
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
                      data-control="ComboBlurType"
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
          <PCard
            data-control="CardFont"
            style="margin: 0px 0px 15px 0px"
            :title="text('Setup.Ui.Font.Title')"
          >
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
                data-control="ComboUiFont"
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
                data-control="ComboUiMotdFont"
                style="grid-row: 3 / span 1; grid-column: 3 / span 1"
                :options="fontOptions"
                :title="text('Setup.Ui.Font.Motd.ToolTip')"
                v-model="values.ComboUiMotdFont"
              />
            </div>
          </PCard>
          <PCard
            data-control="CardBackground"
            style="margin: 0px 0px 15px 0px"
            :title="text('Setup.Ui.Background.TitleDefault')"
          >
            <div style="margin: 0px 25px 20px 25px; display: flex; flex-direction: column">
              <template v-if="hasBackground">
                <div data-control="PanBackgroundSuit" class="pcl-form-row">
                  <label>{{ text('Setup.Ui.Background.ContentAdapt') }}</label
                  ><PComboBox
                    disabled
                    data-control="ComboBackgroundSuit"
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
                <div data-control="PanBackgroundOpacity" class="pcl-form-row">
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
                <div data-control="PanBackgroundBlur" class="pcl-form-row">
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
                data-control="CheckAutoPauseVideo"
                style="height: 22px; margin: 0px 0px 8px -1px"
                :title="text('Setup.Ui.Background.PauseVideo.ToolTip')"
                id="PageSetupUI-CheckAutoPauseVideo"
                :label="text('Setup.Ui.Background.PauseVideo')"
                v-model="values.CheckAutoPauseVideo"
              />
              <PCheckBox
                data-control="CheckBackgroundColorful"
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
                  data-control="BtnBackgroundOpen"
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
                  data-control="BtnBackgroundRefresh"
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
                ><PButton
                  v-if="hasBackground"
                  data-control="BtnBackgroundClear"
                  type="warn"
                  @click="showUnavailable"
                  >{{ text('Setup.Ui.Background.Clear') }}</PButton
                >
              </div>
            </div>
          </PCard>
          <PCard
            data-control="CardMusic"
            style="margin: 0px 0px 15px 0px"
            :title="text('Setup.Ui.Music.Title')"
          >
            <div style="margin: 2px 25px 20px 25px; display: flex; flex-direction: column">
              <template v-if="hasMusic">
                <div data-control="PanMusicVolume" class="pcl-form-row">
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
                <div
                  data-control="PanMusicDetail"
                  style="display: flex; flex-direction: column; gap: 5px; margin-bottom: 8px"
                >
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
                  data-control="BtnMusicOpen"
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
                  data-control="BtnMusicRefresh"
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
                ><PButton
                  v-if="hasMusic"
                  data-control="BtnMusicClear"
                  type="warn"
                  @click="showUnavailable"
                  >{{ text('Setup.Ui.Music.Clear') }}</PButton
                >
              </div>
            </div>
          </PCard>
          <PCard
            data-control="CardLogo"
            style="margin: 0px 0px 15px 0px"
            :title="text('Setup.Ui.Logo.Title')"
          >
            <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
              <div
                style="
                  margin: 0px 0px 0px -1px;
                  display: grid;
                  grid-template-columns:
                    minmax(0, 1fr) minmax(0, 1fr) minmax(0, 1fr) minmax(0, 1fr)
                    minmax(0, 0.2fr);
                "
              >
                <label
                  data-control="RadioLogoType0"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 1 / span 1"
                  class="pcl-radio"
                  ><input disabled type="radio" v-model="values['UiLogoType']" value="0" />{{
                    text('Setup.Ui.Logo.None')
                  }}</label
                >
                <label
                  data-control="RadioLogoType1"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 2 / span 1"
                  class="pcl-radio"
                  ><input disabled type="radio" v-model="values['UiLogoType']" value="1" />{{
                    text('Common.Option.Default')
                  }}</label
                >
                <label
                  data-control="RadioLogoType2"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 3 / span 1"
                  class="pcl-radio"
                  ><input disabled type="radio" v-model="values['UiLogoType']" value="2" />{{
                    text('Setup.Ui.Logo.Text')
                  }}</label
                >
                <label
                  data-control="RadioLogoType3"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 4 / span 1"
                  class="pcl-radio"
                  ><input disabled type="radio" v-model="values['UiLogoType']" value="3" />{{
                    text('Setup.Ui.Logo.Image')
                  }}</label
                >
              </div>
              <PCheckBox
                disabled
                v-if="values.UiLogoType === '0'"
                data-control="CheckLogoLeft"
                style="margin: 8px 0 3px"
                id="PageSetupUI-CheckLogoLeft"
                :label="text('Setup.Ui.Logo.LeftAlign')"
                v-model="values.CheckLogoLeft"
              />
              <div
                data-control="PanLogoText"
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
                  data-control="TextLogoText"
                  style="grid-row: 1 / span 1; grid-column: 2 / span 1"
                  :placeholder="text('')"
                  v-model="values.TextLogoText"
                />
              </div>
              <div
                data-control="PanLogoChange"
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
                  data-control="BtnLogoChange"
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
                  data-control="BtnLogoDelete"
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
          <PCard
            data-control="CardCustom"
            style="margin: 0px 0px 15px 0px"
            :title="text('Setup.Ui.Homepage.Title')"
          >
            <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
              <div
                style="
                  margin: 0px 0px 0px -1px;
                  display: grid;
                  grid-template-columns:
                    minmax(0, 1fr) minmax(0, 1fr) minmax(0, 1fr) minmax(0, 1fr)
                    minmax(0, 0.2fr);
                "
              >
                <label
                  data-control="RadioCustomType0"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 1 / span 1"
                  class="pcl-radio"
                  ><input disabled type="radio" v-model="values['UiCustomType']" value="0" />{{
                    text('Setup.Ui.Homepage.Blank')
                  }}</label
                >
                <label
                  data-control="RadioCustomType3"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 2 / span 1"
                  class="pcl-radio"
                  ><input disabled type="radio" v-model="values['UiCustomType']" value="3" />{{
                    text('Setup.Ui.Homepage.Preset')
                  }}</label
                >
                <label
                  data-control="RadioCustomType1"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 3 / span 1"
                  class="pcl-radio"
                  ><input disabled type="radio" v-model="values['UiCustomType']" value="1" />{{
                    text('Setup.Ui.Homepage.LocalFile')
                  }}</label
                >
                <label
                  data-control="RadioCustomType2"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 4 / span 1"
                  class="pcl-radio"
                  ><input disabled type="radio" v-model="values['UiCustomType']" value="2" />{{
                    text('Setup.Ui.Homepage.NetUpdate')
                  }}</label
                >
              </div>
              <PHint data-control="HintCustom" style="margin: 15px 0px -4px 0px" severity="info">{{
                text('Setup.Ui.Homepage.HintReferToModSetup')
              }}</PHint>
              <PHint
                v-if="values.UiCustomType === '2'"
                data-control="HintCustomWarn"
                style="margin: 15px 0px 6px 0px"
                severity="warning"
                closable
                >{{ text('Setup.Ui.Homepage.NetWarning') }}</PHint
              >
              <div
                data-control="PanCustomLocal"
                style="
                  height: 35px;
                  margin: 10px 0px 5px 0px;
                  display: grid;
                  grid-template-columns: auto auto auto;
                "
                v-if="values.UiCustomType === '1'"
              >
                <PButton
                  data-control="BtnCustomRefresh"
                  style="
                    min-width: 140px;
                    margin: 0px 20px 0px 0px;
                    padding: 0px 13px;
                    grid-row: 1 / span 1;
                    grid-column: 1 / span 1;
                    justify-self: start;
                  "
                  @click="() => router.push('/home')"
                  type="tint"
                  >{{ text('Setup.Ui.Homepage.Refresh') }}</PButton
                >
                <PButton
                  data-control="BtnCustomTutorial"
                  style="
                    min-width: 140px;
                    margin: 0px 20px 0px 0px;
                    padding: 0px 13px;
                    grid-row: 1 / span 1;
                    grid-column: 2 / span 1;
                    justify-self: start;
                  "
                  @click="() => openExternal('https://github.com/Hex-Dragon/PCL2/wiki/自定义主页')"
                  >{{ text('Setup.Ui.Homepage.Tutorial') }}</PButton
                >
              </div>
              <div
                data-control="PanCustomNet"
                style="
                  margin: 10px 0px 5px 0px;
                  display: grid;
                  grid-template-columns: auto minmax(0, 1fr);
                "
                v-if="values.UiCustomType === '2'"
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
                  >{{ text('Setup.Ui.Homepage.DownloadUrl') }}</span
                >
                <PTextBox
                  disabled
                  data-control="TextCustomNet"
                  style="max-height: 28px; grid-row: 1 / span 1; grid-column: 2 / span 1"
                  :placeholder="text('')"
                  v-model="values.TextCustomNet"
                />
              </div>
              <div
                data-control="PanCustomPreset"
                style="
                  height: 28px;
                  margin: 12px 10px 5px 0px;
                  display: grid;
                  grid-template-columns: auto minmax(0, 1fr);
                "
                v-if="values.UiCustomType === '3'"
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
                  >{{ text('Setup.Ui.Homepage.PresetLabel') }}</span
                >
                <PComboBox
                  disabled
                  data-control="ComboCustomPreset"
                  style="grid-row: 1 / span 1; grid-column: 2 / span 2"
                  :options="[
                    { key: '0', text: text('Setup.Ui.Homepage.Preset.Trivia') },
                    { key: '1', text: text('Setup.Ui.Homepage.Preset.McNews') },
                    { key: '2', text: text('Setup.Ui.Homepage.Preset.DailyModpack') },
                    { key: '3', text: text('Setup.Ui.Homepage.Preset.McSkin') },
                    { key: '4', text: text('Setup.Ui.Homepage.Preset.OpenBmclapi') },
                    { key: '5', text: text('Setup.Ui.Homepage.Preset.PclManual') },
                    { key: '6', text: text('Setup.Ui.Homepage.Preset.Magazine') },
                    { key: '7', text: text('Setup.Ui.Homepage.Preset.GitHub') },
                    { key: '8', text: text('Setup.Ui.Homepage.Preset.McUpdateSummary') },
                    { key: '9', text: text('Setup.Ui.Homepage.Preset.NewsToday') },
                    { key: '10', text: text('Setup.Ui.Homepage.Preset.MinecraftKnowledge') },
                    { key: '11', text: text('Setup.Ui.Homepage.Preset.ModpackRecommendation') },
                    { key: '12', text: text('Setup.Ui.Homepage.Preset.Bangumi') },
                    { key: '13', text: text('Setup.Ui.Homepage.Preset.PclAnnouncement') },
                    { key: '14', text: text('Setup.Ui.Homepage.Preset.McOfficialFeed') },
                  ]"
                  v-model="values.ComboCustomPreset"
                />
              </div>
            </div>
          </PCard>
          <PCard
            data-control="CardSwitch"
            style="margin: 0px 0px 15px 0px"
            :title="text('Setup.Ui.FeatureHide.Title')"
          >
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
                  data-control="CheckHiddenPageDownload"
                  style="
                    height: 22px;
                    grid-row: 2 / span 1;
                    grid-column: 3 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenPageDownload"
                  :label="text('Setup.Ui.FeatureHide.Item.Download')"
                  v-model="values.CheckHiddenPageDownload"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenPageSetup"
                  style="
                    height: 22px;
                    grid-row: 2 / span 1;
                    grid-column: 4 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenPageSetup"
                  :label="text('Setup.Ui.FeatureHide.Item.Settings')"
                  v-model="values.CheckHiddenPageSetup"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenPageTools"
                  style="
                    height: 22px;
                    grid-row: 2 / span 1;
                    grid-column: 5 / span 1;
                    align-self: center;
                  "
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
                  data-control="CheckHiddenSetupLaunch"
                  style="
                    height: 22px;
                    grid-row: 3 / span 1;
                    grid-column: 3 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenSetupLaunch"
                  :label="text('Setup.Ui.FeatureHide.Item.Launch')"
                  v-model="values.CheckHiddenSetupLaunch"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenSetupJava"
                  style="
                    height: 22px;
                    grid-row: 3 / span 1;
                    grid-column: 4 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenSetupJava"
                  :label="text('Java')"
                  v-model="values.CheckHiddenSetupJava"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenSetupGameManage"
                  style="
                    height: 22px;
                    grid-row: 3 / span 1;
                    grid-column: 5 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenSetupGameManage"
                  :label="text('Setup.Ui.FeatureHide.Item.GameManage')"
                  v-model="values.CheckHiddenSetupGameManage"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenSetupGameLink"
                  style="
                    height: 22px;
                    grid-row: 3 / span 1;
                    grid-column: 6 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenSetupGameLink"
                  :label="text('Setup.Ui.FeatureHide.Item.GameLink')"
                  v-model="values.CheckHiddenSetupGameLink"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenSetupUI"
                  style="
                    height: 22px;
                    grid-row: 3 / span 1;
                    grid-column: 7 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenSetupUI"
                  :label="text('Setup.Ui.FeatureHide.Item.Ui')"
                  v-model="values.CheckHiddenSetupUI"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenSetupLauncherLanguage"
                  style="
                    height: 22px;
                    grid-row: 4 / span 1;
                    grid-column: 3 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenSetupLauncherLanguage"
                  :label="text('Setup.Ui.FeatureHide.Item.Language')"
                  v-model="values.CheckHiddenSetupLauncherLanguage"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenLauncherMisc"
                  style="
                    height: 22px;
                    grid-row: 4 / span 1;
                    grid-column: 4 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenLauncherMisc"
                  :label="text('Setup.Ui.FeatureHide.Item.Misc')"
                  v-model="values.CheckHiddenLauncherMisc"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenSetupUpdate"
                  style="
                    height: 22px;
                    grid-row: 4 / span 1;
                    grid-column: 5 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenSetupUpdate"
                  :label="text('Setup.Ui.FeatureHide.Item.Update')"
                  v-model="values.CheckHiddenSetupUpdate"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenSetupAbout"
                  style="
                    height: 22px;
                    grid-row: 4 / span 1;
                    grid-column: 6 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenSetupAbout"
                  :label="text('Setup.Ui.FeatureHide.Item.About')"
                  v-model="values.CheckHiddenSetupAbout"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenSetupFeedback"
                  style="
                    height: 22px;
                    grid-row: 4 / span 1;
                    grid-column: 7 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenSetupFeedback"
                  :label="text('Setup.Ui.FeatureHide.Item.Feedback')"
                  v-model="values.CheckHiddenSetupFeedback"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenSetupLog"
                  style="
                    height: 22px;
                    grid-row: 4 / span 1;
                    grid-column: 8 / span 1;
                    align-self: center;
                  "
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
                  data-control="CheckHiddenToolsGameLink"
                  style="
                    height: 22px;
                    grid-row: 5 / span 1;
                    grid-column: 3 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenToolsGameLink"
                  :label="text('Setup.Ui.FeatureHide.Item.Lobby')"
                  v-model="values.CheckHiddenToolsGameLink"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenToolsTest"
                  style="
                    height: 22px;
                    grid-row: 5 / span 1;
                    grid-column: 4 / span 1;
                    align-self: center;
                  "
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
                  data-control="CheckHiddenVersionEdit"
                  style="
                    height: 22px;
                    grid-row: 6 / span 1;
                    grid-column: 3 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenVersionEdit"
                  :label="text('Setup.Ui.FeatureHide.Item.Edit')"
                  v-model="values.CheckHiddenVersionEdit"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenVersionExport"
                  style="
                    height: 22px;
                    grid-row: 6 / span 1;
                    grid-column: 4 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenVersionExport"
                  :label="text('Setup.Ui.FeatureHide.Item.Export')"
                  v-model="values.CheckHiddenVersionExport"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenVersionSave"
                  style="
                    height: 22px;
                    grid-row: 6 / span 1;
                    grid-column: 5 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenVersionSave"
                  :label="text('Setup.Ui.FeatureHide.Item.Save')"
                  v-model="values.CheckHiddenVersionSave"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenVersionScreenshot"
                  style="
                    height: 22px;
                    grid-row: 6 / span 1;
                    grid-column: 6 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenVersionScreenshot"
                  :label="text('Setup.Ui.FeatureHide.Item.Screenshot')"
                  v-model="values.CheckHiddenVersionScreenshot"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenVersionMod"
                  style="
                    height: 22px;
                    grid-row: 6 / span 1;
                    grid-column: 7 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenVersionMod"
                  :label="text('Mod')"
                  v-model="values.CheckHiddenVersionMod"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenVersionResourcePack"
                  style="
                    height: 22px;
                    grid-row: 7 / span 1;
                    grid-column: 3 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenVersionResourcePack"
                  :label="text('Setup.Ui.FeatureHide.Item.ResourcePack')"
                  v-model="values.CheckHiddenVersionResourcePack"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenVersionShader"
                  style="
                    height: 22px;
                    grid-row: 7 / span 1;
                    grid-column: 4 / span 1;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenVersionShader"
                  :label="text('Setup.Ui.FeatureHide.Item.Shader')"
                  v-model="values.CheckHiddenVersionShader"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenVersionSchematic"
                  style="
                    height: 22px;
                    grid-row: 7 / span 1;
                    grid-column: 5 / span 2;
                    align-self: center;
                  "
                  id="PageSetupUI-CheckHiddenVersionSchematic"
                  :label="text('Setup.Ui.FeatureHide.Item.Schematic')"
                  v-model="values.CheckHiddenVersionSchematic"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenVersionServer"
                  style="
                    height: 22px;
                    grid-row: 7 / span 1;
                    grid-column: 7 / span 1;
                    align-self: center;
                  "
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
                  data-control="CheckHiddenFunctionSelect"
                  style="
                    height: 22px;
                    grid-row: 8 / span 1;
                    grid-column: 3 / span 1;
                    align-self: center;
                  "
                  :title="text('Setup.Ui.FeatureHide.Item.InstanceSelect.ToolTip')"
                  id="PageSetupUI-CheckHiddenFunctionSelect"
                  :label="text('Setup.Ui.FeatureHide.Item.InstanceSelect')"
                  v-model="values.CheckHiddenFunctionSelect"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenFunctionModUpdate"
                  style="
                    height: 22px;
                    grid-row: 8 / span 1;
                    grid-column: 4 / span 1;
                    align-self: center;
                  "
                  :title="text('Setup.Ui.FeatureHide.Item.ModUpdate.ToolTip')"
                  id="PageSetupUI-CheckHiddenFunctionModUpdate"
                  :label="text('Setup.Ui.FeatureHide.Item.ModUpdate')"
                  v-model="values.CheckHiddenFunctionModUpdate"
                />
                <PCheckBox
                  disabled
                  data-control="CheckHiddenFunctionHidden"
                  style="
                    height: 22px;
                    grid-row: 8 / span 1;
                    grid-column: 5 / span 1;
                    align-self: center;
                  "
                  :title="text('Setup.Ui.FeatureHide.Item.FeatureHide.ToolTip')"
                  id="PageSetupUI-CheckHiddenFunctionHidden"
                  :label="text('Setup.Ui.FeatureHide.Item.FeatureHide')"
                  v-model="values.CheckHiddenFunctionHidden"
                />
              </div>
            </div>
          </PCard>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.logo-image-actions {
  justify-content: start;
}
</style>
