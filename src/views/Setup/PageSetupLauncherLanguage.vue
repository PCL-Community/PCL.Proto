<script setup lang="ts">
import { computed } from 'vue'
import PIconTextButton from '@/components/widget/PIconTextButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import {
  pclLanguagePreference,
  pclFormatPreference,
  pclLanguages,
  pclText,
  resolveSystemLanguage,
  setPclLocale,
  setPclFormatLocale,
} from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import { openExternal } from '@/util/externalLink'
const languageName = (key: string) =>
  new Intl.DisplayNames([key], { type: 'language' }).of(key) || key
const languageOptions = computed(() => [
  {
    key: 'auto',
    text: pclText('Setup.Language.UiLanguage.Auto', [languageName(resolveSystemLanguage())]),
  },
  ...pclLanguages.map((key) => ({
    key,
    text: languageName(key),
  })),
])
const formatOptions = computed(() => [
  { key: 'auto', text: pclText('Setup.Language.FormatCulture.Auto') },
  { key: 'ui-language', text: pclText('Setup.Language.FormatCulture.FollowLanguage') },
  ...pclLanguages.map((key) => ({ key, text: languageName(key) })),
])
async function setLanguage(value?: string) {
  if (!value) return
  try {
    await setPclLocale(value)
  } catch (error) {
    sideTip.show(pclText('Proto.Language.Failed', [String(error)]), 'warn')
  }
}
const text = pclText
</script>
<template>
  <div class="pcl-page">
    <PCard :title="text('Setup.Language.CardTitle')">
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
            style="grid-row: 1 / span 1; grid-column: 2 / span 2"
            :options="languageOptions"
            :model-value="pclLanguagePreference"
            :aria-label="text('Setup.Language.UiLanguage')"
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
            style="grid-row: 3 / span 1; grid-column: 2 / span 2"
            :options="formatOptions"
            :model-value="pclFormatPreference"
            :aria-label="text('Setup.Language.FormatCulture')"
            @update:model-value="(value) => value && setPclFormatLocale(value)"
          />
        </div>
        <div
          class="internationalization-banner"
          style="min-height: 230px; margin: 10px 0px 0px 0px; padding: 25px; border-radius: 5px"
        >
          <div style="display: grid">
            <PSvgIcon class="internationalization-icon" name="lucide/earth" />
            <div
              class="internationalization-content"
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
                <span style="font-size: 22px; font-weight: 700" class="pcl-text">{{
                  text('Setup.Language.Banner.TitlePrefix')
                }}</span>
                <span
                  style="margin: 2px 0px 2px 0px; font-size: 42px; font-weight: 700"
                  class="pcl-text"
                  >{{ 'PCL.Proto' }}</span
                >
                <span style="font-size: 22px; font-weight: 700" class="pcl-text">{{
                  text('Setup.Language.Banner.TitleSuffix')
                }}</span>
              </div>
              <div
                class="internationalization-description"
                style="
                  margin: 20px 0px 0px 0px;
                  grid-row: 2 / span 1;
                  grid-column: 1 / span 1;
                  display: flex;
                  flex-direction: column;
                "
              >
                <span style="font-size: 14px" class="pcl-text">{{
                  text('Proto.Language.Banner.DescriptionLine1', ['PCL CE'])
                }}</span>
                <span style="margin: 5px 0px 0px 0px; font-size: 14px" class="pcl-text">{{
                  text('Proto.Language.Banner.DescriptionLine2', ['PCL CE'])
                }}</span>
                <span style="margin: 5px 0px 0px 0px; font-size: 14px" class="pcl-text">{{
                  text('Proto.Language.Banner.DescriptionLine3', ['PCL CE'])
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
            justify-content: flex-end;
            flex-wrap: wrap;
            gap: 7px;
          "
        >
          <PIconTextButton
            icon="lucide/globe"
            class="translation-link"
            @click="() => openExternal('https://translate.pclc.cc/projects/pcl-ce/')"
            >{{ text('Setup.Language.Banner.TranslateWebsite') }}</PIconTextButton
          >
          <PIconTextButton
            icon="lucide/git-pull-request-arrow"
            @click="() => openExternal('https://github.com/PCL-Community/PCL.Proto/pulls')"
            >{{ text('Setup.Language.Banner.SubmitPullRequest') }}</PIconTextButton
          >
        </div>
      </div>
    </PCard>
  </div>
</template>

<style scoped>
.translation-link {
  color: var(--color-tint);
}
.internationalization-banner {
  position: relative;
  overflow: hidden;
  background: var(--color-tint-lighter);
}
.internationalization-icon {
  position: absolute;
  top: -74px;
  right: -15px;
  width: 215px;
  height: 215px;
  color: var(--ce-color-5, var(--color-tint-light));
  opacity: 0.62;
  pointer-events: none;
  stroke-width: 2;
}
.internationalization-content {
  position: relative;
}
.internationalization-description {
  line-height: 23px;
}
</style>
