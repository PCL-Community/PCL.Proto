<script setup lang="ts">
import { usePageSettings } from '@/composables/usePageSettings'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import { pclLocale as locale, pclLanguages, pclText, setPclLocale } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import { openExternal } from '@/util/externalLink'
const languageOptions = pclLanguages.map((key) => ({
  key,
  text: new Intl.DisplayNames([key], { type: 'language' }).of(key) || key,
}))
async function setLanguage(value?: string) {
  if (!value) return
  try {
    await setPclLocale(value)
  } catch (error) {
    sideTip.show(pclText('Proto.Language.Failed', [String(error)]), 'warn')
  }
}
const { values } = usePageSettings('PageSetupLauncherLanguage', {
  ComboUiLanguage: '0',
  ComboUiFormatCulture: '0',
})

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
          <PCard style="margin: 15px 0px 10px 0px" :title="text('Setup.Language.CardTitle')">
            <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
              <div
                style="
                  display: grid;
                  grid-template-columns: auto minmax(0, 1fr) 60px;
                  grid-template-rows: 28px 9px 28px;
                "
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
                  >{{ text('Setup.Language.UiLanguage') }}</span
                >
                <PComboBox
                  data-control="ComboUiLanguage"
                  style="grid-row: 1 / span 1; grid-column: 2 / span 2"
                  :options="languageOptions"
                  :model-value="locale"
                  @update:model-value="setLanguage"
                />
                <span
                  style="
                    margin: 0px 25px 0px 0px;
                    grid-row: 3 / span 1;
                    grid-column: 1 / span 1;
                    justify-self: start;
                    align-self: center;
                  "
                  class="pcl-text"
                  >{{ text('Setup.Language.FormatCulture') }}</span
                >
                <PComboBox
                  disabled
                  data-control="ComboUiFormatCulture"
                  style="grid-row: 3 / span 1; grid-column: 2 / span 2"
                  :options="[
                    { key: '0', text: text('Setup.Language.FormatCulture.Auto') },
                    { key: '1', text: text('Setup.Language.FormatCulture.FollowLanguage') },
                  ]"
                  v-model="values.ComboUiFormatCulture"
                />
              </div>
              <div
                data-control="BannerInternationalization"
                class="internationalization-banner"
                style="
                  min-height: 230px;
                  margin: 10px 0px 0px 0px;
                  padding: 25px;
                  border-radius: 5px;
                "
              >
                <div style="display: grid">
                  <div
                    style="
                      width: 205px;
                      height: 145px;
                      margin: -43px -25px 0px 0px;
                      grid-row: 1 / span 1;
                      grid-column: 1 / span 1;
                      justify-self: end;
                      align-self: start;
                    "
                  >
                    <PSvgIcon
                      style="width: 215px; height: 215px; opacity: 0.62"
                      name="lucide/earth"
                    />
                  </div>
                  <div
                    style="
                      grid-row: 1 / span 1;
                      grid-column: 1 / span 1;
                      display: grid;
                      grid-template-rows: auto auto minmax(0, 1fr);
                    "
                  >
                    <div
                      style="
                        margin: 0px 170px 0px 0px;
                        grid-row: 1 / span 1;
                        grid-column: 1 / span 1;
                        justify-self: start;
                        display: flex;
                        flex-direction: column;
                      "
                    >
                      <span style="font-size: 22px; font-weight: bold" class="pcl-text">{{
                        text('Setup.Language.Banner.TitlePrefix')
                      }}</span>
                      <span
                        style="margin: 2px 0px 2px 0px; font-size: 42px; font-weight: bold"
                        class="pcl-text"
                        >{{ 'PCL.Proto' }}</span
                      >
                      <span style="font-size: 22px; font-weight: bold" class="pcl-text">{{
                        text('Setup.Language.Banner.TitleSuffix')
                      }}</span>
                    </div>
                    <div
                      style="
                        margin: 20px 0px 0px 0px;
                        grid-row: 2 / span 1;
                        grid-column: 1 / span 1;
                        display: flex;
                        flex-direction: column;
                      "
                    >
                      <span style="font-size: 14px" class="pcl-text">{{
                        text('Setup.Language.Banner.DescriptionLine1')
                      }}</span>
                      <span style="margin: 5px 0px 0px 0px; font-size: 14px" class="pcl-text">{{
                        text('Setup.Language.Banner.DescriptionLine2')
                      }}</span>
                      <span style="margin: 5px 0px 0px 0px; font-size: 14px" class="pcl-text">{{
                        text('Setup.Language.Banner.DescriptionLine3')
                      }}</span>
                    </div>
                  </div>
                </div>
              </div>
              <div
                style="
                  margin: 10px 0px 0px 0px;
                  display: flex;
                  flex-direction: row;
                  align-items: center;
                "
              >
                <PButton
                  data-control="BtnTranslateWebsite"
                  style="margin: 0px 7px 0px 0px"
                  @click="() => openExternal('https://translate.pclc.cc/projects/pcl-ce/')"
                  inline
                  type="tint"
                  ><PSvgIcon name="lucide/globe" :size="18" />{{
                    text('Setup.Language.Banner.TranslateWebsite')
                  }}</PButton
                >
                <PButton
                  data-control="BtnSubmitPullRequest"
                  style="margin: 0px 7px 0px 0px"
                  @click="() => openExternal('https://github.com/PCL-Community/PCL.Proto/pulls')"
                  inline
                  ><PSvgIcon name="lucide/git-pull-request-arrow" :size="18" />{{
                    text('Setup.Language.Banner.SubmitPullRequest')
                  }}</PButton
                >
              </div>
            </div>
          </PCard>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.internationalization-banner {
  background: var(--color-background-soft);
  box-shadow: var(--box-shadow);
}
</style>
