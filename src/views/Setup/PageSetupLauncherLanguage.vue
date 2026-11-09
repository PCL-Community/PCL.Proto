<script setup lang="ts">
import SetupItem from '@/components/widget/SetupItem.vue'
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
</script>
<template>
  <div class="pcl-page">
    <PCard :title="pclText('Setup.Language.CardTitle')">
      <div class="pcl-card-body">
        <div class="settings-grid">
          <SetupItem :label="pclText('Setup.Language.UiLanguage')" v-slot="{ id }">
            <PComboBox
              :input-id="id"
              :options="languageOptions"
              :model-value="pclLanguagePreference"
              @update:model-value="setLanguage"
            />
          </SetupItem>
          <SetupItem :label="pclText('Setup.Language.FormatCulture')" v-slot="{ id }">
            <PComboBox
              :input-id="id"
              :options="formatOptions"
              :model-value="pclFormatPreference"
              @update:model-value="(value) => value && setPclFormatLocale(value)"
            />
          </SetupItem>
        </div>
        <div
          class="internationalization-banner"
          style="min-height: 230px; padding: 25px; border-radius: 5px"
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
                  pclText('Setup.Language.Banner.TitlePrefix')
                }}</span>
                <span style="font-size: 42px; font-weight: 700" class="pcl-text">{{
                  'PCL.Proto'
                }}</span>
                <span style="font-size: 22px; font-weight: 700" class="pcl-text">{{
                  pclText('Setup.Language.Banner.TitleSuffix')
                }}</span>
              </div>
              <div
                class="internationalization-description"
                style="
                  grid-row: 2 / span 1;
                  grid-column: 1 / span 1;
                  display: flex;
                  flex-direction: column;
                "
              >
                <span style="font-size: 14px" class="pcl-text">{{
                  pclText('Proto.Language.Banner.DescriptionLine1', ['PCL CE'])
                }}</span>
                <span style="font-size: 14px" class="pcl-text">{{
                  pclText('Proto.Language.Banner.DescriptionLine2', ['PCL CE'])
                }}</span>
                <span style="font-size: 14px" class="pcl-text">{{
                  pclText('Proto.Language.Banner.DescriptionLine3', ['PCL CE'])
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
            >{{ pclText('Setup.Language.Banner.TranslateWebsite') }}</PIconTextButton
          >
          <PIconTextButton
            icon="lucide/git-pull-request-arrow"
            @click="() => openExternal('https://github.com/PCL-Community/PCL.Proto/pulls')"
            >{{ pclText('Setup.Language.Banner.SubmitPullRequest') }}</PIconTextButton
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
  margin-top: 10px;
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
  margin-top: 20px;
  gap: 5px;
  line-height: 23px;
}
</style>
