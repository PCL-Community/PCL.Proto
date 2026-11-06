<script setup lang="ts">
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import { ref, watch, defineAsyncComponent } from 'vue'
import { useRouter } from 'vue-router'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { basename, dirname, join } from '@tauri-apps/api/path'
import { open, save } from '@tauri-apps/plugin-dialog'
import { isTaskCancelled, useTaskManager } from '@/stores/task'
import { usePageSettings } from '@/composables/usePageSettings'
import { useModal } from '@/composables/useModal'
import sideTip from '@/composables/sideTip'
import getSkinUrl from '@/api/skinGetter'
import MinecraftServerCard from '@/components/widget/MinecraftServerCard.vue'
const savingSkin = ref(false)
const loadingSkin = ref(false)
const skin = ref<{ name: string; url: string }>()
const SkinViewer = defineAsyncComponent(() => import('@/components/widget/SkinViewer.vue'))
const router = useRouter()
import PTextBox from '@/components/widget/PTextBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PCard from '@/components/widget/PCard.vue'
import PButton from '@/components/widget/PButton.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
const { values } = usePageSettings('PageToolsTest', {
  TextSkinID: '',
  TextDownloadUrl: '',
  TextDownloadFolder: '',
  TextDownloadName: '',
  AchievementBlockTextBox: '',
  AchievementTitleTextBox: '',
  AchievementString1TextBox: '',
  AchievementString2TextBox: '',
  CmbHeadSize: '0',
})
async function showLuck() {
  try {
    const luck = await invoke<number>('get_lucky_today')
    await useModal().open({
      title: pclText('Tools.Test.Luck.Title'),
      content: pclText('Proto.Tools.Luck', [luck]),
      buttons: [{ type: 'tint', content: pclText('Common.Action.Confirm') }],
    })
  } catch (err) {
    sideTip.show(pclText('Proto.Tools.LuckFailed', [String(err)]), 'warn')
  }
}
async function loadSkin() {
  if (loadingSkin.value || savingSkin.value) return
  const name = String(values.TextSkinID || '').trim()
  if (!/^[A-Za-z0-9_]{3,16}$/.test(name)) {
    sideTip.show(pclText('Proto.Tools.InvalidPlayer'), 'warn')
    return
  }
  if (!isTauri()) return sideTip.show(pclText('Proto.Desktop.Download'), 'warn')
  loadingSkin.value = true
  try {
    let skinUrl: string | undefined
    await getSkinUrl(name, (url) => (skinUrl = url), 'username')
    if (!skinUrl) throw new Error('Skin is unavailable')
    skin.value = { name, url: skinUrl }
  } catch (err) {
    sideTip.show(pclText('Proto.Tools.SkinFailed', [String(err)]), 'warn')
  } finally {
    loadingSkin.value = false
  }
}

async function saveSkin() {
  const current = skin.value
  if (!current || savingSkin.value || loadingSkin.value) return
  savingSkin.value = true
  try {
    const response = await fetch(current.url)
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    const bytes = new Uint8Array(await response.arrayBuffer())
    const chunks: string[] = []
    for (let index = 0; index < bytes.length; index += 0x8000) {
      chunks.push(String.fromCharCode(...bytes.subarray(index, index + 0x8000)))
    }
    const destination = await save({
      title: pclText('Tools.Test.Skin.Save'),
      defaultPath: `${current.name}.png`,
      filters: [{ name: pclText('Launch.Skin.SaveDialog.Filter'), extensions: ['png'] }],
    })
    if (!destination) return
    await invoke('save_profile_skin', { destination, image: btoa(chunks.join('')) })
    sideTip.show(pclText('Tools.Test.Skin.Saved', [current.name]), 'success')
  } catch (err) {
    sideTip.show(pclText('Launch.Skin.Save.Error'), 'warn')
  } finally {
    savingSkin.value = false
  }
}

watch(
  () => values.TextDownloadUrl,
  (value) => {
    if (values.TextDownloadName || !value) return
    try {
      const name = new URL(String(value)).pathname.split('/').pop()
      if (name) values.TextDownloadName = decodeURIComponent(name)
    } catch {
      /* Wait for a complete URL. */
    }
  },
)

async function chooseDownloadFolder() {
  if (!isTauri()) return sideTip.show(pclText('Proto.Desktop.Download'), 'warn')
  try {
    const folder = await open({
      directory: true,
      defaultPath: String(values.TextDownloadFolder || '') || undefined,
    })
    if (folder) values.TextDownloadFolder = folder
  } catch (error) {
    sideTip.show(pclText('Proto.Error.Download', [String(error)]), 'warn')
  }
}

async function startCustomDownload() {
  if (!isTauri()) return sideTip.show(pclText('Proto.Desktop.Download'), 'warn')
  try {
    const url = new URL(String(values.TextDownloadUrl || '').trim())
    if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password)
      throw new Error('Invalid download URL')
    let name = String(values.TextDownloadName || '').trim()
    if (!name || /[\\/:*?"<>|\x00-\x1f]/.test(name) || name === '.' || name === '..')
      throw new Error('Invalid filename')
    const folder = String(values.TextDownloadFolder || '').trim()
    const destination = folder
      ? await join(folder, name)
      : await save({
          defaultPath: name,
          title: pclText('Tools.Test.CustomDownload.SelectLocation'),
        })
    if (!destination) return
    if (!folder) {
      values.TextDownloadFolder = await dirname(destination)
      name = await basename(destination)
    }
    const task = useTaskManager().downloadCustom(name, url.href, destination)
    values.TextDownloadUrl = ''
    values.TextDownloadName = ''
    sideTip.show(pclText('Download.Comp.QuickDownload.Hint.DownloadStarted', [name]), 'success')
    void task
      .then(() => sideTip.show(pclText('Tools.Test.CustomDownload.Finished', [name]), 'success'))
      .catch((error) => {
        if (!isTaskCancelled(error))
          sideTip.show(pclText('Proto.Error.Download', [String(error)]), 'warn')
      })
  } catch (error) {
    sideTip.show(pclText('Proto.Error.Download', [String(error)]), 'warn')
  }
}

async function openDownloadFolder() {
  if (!isTauri()) return sideTip.show(pclText('Proto.Desktop.Download'), 'warn')
  try {
    await invoke('open_custom_download_folder', {
      directory: String(values.TextDownloadFolder || '').trim(),
    })
  } catch (error) {
    sideTip.show(pclText('Proto.Error.Download', [String(error)]), 'warn')
  }
}
</script>
<template>
  <section class="pcl-page">
    <PCard :title="pclText('Tools.Test.Title')">
      <div
        style="
          margin: 0px 21px 5px 25px;
          display: flex;
          flex-direction: row;
          align-items: center;
          flex-wrap: wrap;
        "
      >
        <PButton
          style="min-width: 120px; margin: 0px 15px 15px 0px"
          @click="() => router.push('/homepageeditor')"
          >{{ pclText('Proto.Title.HomepageEditor') }}</PButton
        >
        <PButton
          style="min-width: 120px; margin: 0px 15px 15px 0px"
          :title="pclText('Tools.Test.Clean.ToolTip')"
          @click="showUnavailable"
          >{{ pclText('Tools.Test.Clean.Title') }}</PButton
        >
        <PButton style="min-width: 100px; margin: 0px 15px 15px 0px" @click="showLuck">{{
          pclText('Tools.Test.Luck.Title')
        }}</PButton>
        <PButton
          style="min-width: 120px; margin: 0px 15px 15px 0px"
          :title="pclText('Tools.Test.Shortcut.ToolTip')"
          @click="showUnavailable"
          >{{ pclText('Tools.Test.Shortcut.Title') }}</PButton
        >
        <PButton style="min-width: 120px; margin: 0px 15px 15px 0px" @click="showUnavailable">{{
          pclText('Tools.Test.LaunchCount.Title')
        }}</PButton>
      </div>
    </PCard>
    <PCard :title="pclText('Tools.Test.CustomDownload.Title')">
      <div style="margin: 0px 25px 20px 25px; display: flex; flex-direction: column">
        <span class="pcl-text">{{ pclText('Proto.Tools.CustomDownloadDescription') }}</span>
        <div
          style="
            margin: 10px 0px 0px 0px;
            display: grid;
            grid-template-columns: auto minmax(0, 1fr) auto;
            grid-template-rows: auto auto auto minmax(0, 1fr);
          "
        >
          <span
            style="
              margin: 0px 25px 0px 0px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
              align-self: center;
            "
            class="pcl-text"
            >{{ pclText('Tools.Test.CustomDownload.Url') }}</span
          >
          <PTextBox
            :aria-label="pclText('Tools.Test.CustomDownload.Url')"
            style="grid-row: 1 / span 1; grid-column: 2 / span 2"
            :placeholder="pclText('')"
            v-model="values.TextDownloadUrl"
          />
          <span
            style="
              margin: 0px 25px 0px 0px;
              grid-row: 3 / span 1;
              grid-column: 1 / span 1;
              align-self: center;
            "
            class="pcl-text"
            >{{ pclText('Tools.Test.CustomDownload.SaveTo') }}</span
          >
          <PTextBox
            :aria-label="pclText('Tools.Test.CustomDownload.SaveTo')"
            style="margin: 10px 0px; grid-row: 3 / span 1; grid-column: 2 / span 1"
            :placeholder="pclText('')"
            v-model="values.TextDownloadFolder"
          />
          <PButton
            style="
              margin: 0px 5px 0px 10px;
              grid-row: 3 / span 1;
              grid-column: 3 / span 1;
              align-self: center;
            "
            @click="chooseDownloadFolder"
            inline
            >{{ pclText('Tools.Test.CustomDownload.Select') }}</PButton
          >
          <span
            style="
              margin: 0px 25px 0px 0px;
              grid-row: 4 / span 1;
              grid-column: 1 / span 1;
              align-self: center;
            "
            class="pcl-text"
            >{{ pclText('Tools.Test.CustomDownload.FileName') }}</span
          >
          <PTextBox
            :aria-label="pclText('Tools.Test.CustomDownload.FileName')"
            style="grid-row: 4 / span 1; grid-column: 2 / span 2"
            :placeholder="pclText('')"
            v-model="values.TextDownloadName"
          />
        </div>
        <div
          style="
            margin: 15px 0px 0px 0px;
            display: grid;
            grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
            grid-template-rows: 35px;
          "
        >
          <PButton
            style="
              width: 140px;
              margin: 0px 15px 0px 0px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
              justify-self: end;
            "
            @click="startCustomDownload"
            type="tint"
            >{{ pclText('Tools.Test.CustomDownload.Start') }}</PButton
          >
          <PButton
            style="
              width: 140px;
              margin: 0px 15px 0px 0px;
              grid-row: 1 / span 1;
              grid-column: 2 / span 1;
              justify-self: start;
            "
            @click="openDownloadFolder"
            >{{ pclText('Common.Action.OpenFolder') }}</PButton
          >
        </div>
      </div>
    </PCard>
    <PCard :title="pclText('Tools.Test.Skin.Title')">
      <div class="skin-tool">
        <div class="skin-layout" :class="{ 'has-skin': skin }">
          <div class="skin-controls">
            <label for="skin-player-name">{{ pclText('Tools.Test.Skin.PlayerName') }}</label>
            <div class="skin-query">
              <PTextBox
                input-id="skin-player-name"
                :model-value="String(values.TextSkinID || '')"
                :disabled="loadingSkin || savingSkin"
                @update:model-value="(value) => (values.TextSkinID = value || '')"
                @keydown.enter="loadSkin"
              />
              <PButton
                compact
                :disabled="loadingSkin || savingSkin"
                :aria-busy="loadingSkin"
                @click="loadSkin"
              >
                {{ pclText('Tools.ServerQuery.Action.Query') }}
              </PButton>
            </div>
            <div v-if="loadingSkin" class="skin-status" role="status">
              {{ pclText('Tools.Test.Skin.Fetching') }}
            </div>
            <Transition name="skin-reveal">
              <div v-if="skin" class="skin-result-actions">
                <span class="skin-result-name"
                  >{{ pclText('Proto.Label.PlayerSkin') }}: <strong>{{ skin.name }}</strong></span
                >
                <PButton compact :disabled="loadingSkin || savingSkin" @click="saveSkin">
                  {{ pclText('Tools.Test.Skin.Save') }}
                </PButton>
                <button
                  class="pcl-icon-button"
                  type="button"
                  :title="pclText('Proto.Tools.CloseSkinPreview')"
                  :aria-label="pclText('Proto.Tools.CloseSkinPreview')"
                  :disabled="loadingSkin || savingSkin"
                  @click="skin = undefined"
                >
                  <PSvgIcon name="lucide/x" :size="18" />
                </button>
              </div>
            </Transition>
          </div>
          <div class="skin-preview">
            <Transition name="skin-reveal">
              <div v-if="skin" class="skin-preview-content">
                <SkinViewer :skin-url="skin.url" :aria-label="pclText('Proto.Label.PlayerSkin')" />
              </div>
            </Transition>
          </div>
        </div>
      </div>
    </PCard>
    <PCard :title="pclText('Tools.Test.ServerQuery.Title')">
      <div style="margin: 0px 15px 15px 15px"><MinecraftServerCard embedded /></div>
    </PCard>
    <PCard :title="pclText('Tools.Test.Achievement.Title')" default-fold-status="fold">
      <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
        <div
          style="
            margin: 4px 0px 2px 0px;
            display: grid;
            grid-template-columns: auto minmax(0, 1fr) auto;
            grid-template-rows: auto auto auto auto minmax(0, 1fr);
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
            >{{ pclText('Tools.Test.Achievement.ItemId') }}</span
          >
          <PTextBox
            style="margin: 10px 0px; grid-row: 1 / span 1; grid-column: 2 / span 2"
            :placeholder="pclText('')"
            v-model="values.AchievementBlockTextBox"
          />
          <span
            style="
              margin: 0px 25px 0px 0px;
              grid-row: 2 / span 1;
              grid-column: 1 / span 1;
              justify-self: start;
              align-self: center;
            "
            class="pcl-text"
            >{{ pclText('Tools.Test.Achievement.Name') }}</span
          >
          <PTextBox
            style="grid-row: 2 / span 1; grid-column: 2 / span 2"
            :placeholder="pclText('')"
            v-model="values.AchievementTitleTextBox"
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
            >{{ pclText('Tools.Test.Achievement.Line1') }}</span
          >
          <PTextBox
            style="margin: 10px 0px; grid-row: 3 / span 1; grid-column: 2 / span 2"
            :placeholder="pclText('')"
            v-model="values.AchievementString1TextBox"
          />
          <span
            style="
              margin: 0px 25px 0px 0px;
              grid-row: 4 / span 1;
              grid-column: 1 / span 1;
              justify-self: start;
              align-self: center;
            "
            class="pcl-text"
            >{{ pclText('Tools.Test.Achievement.Line2') }}</span
          >
          <PTextBox
            style="grid-row: 4 / span 1; grid-column: 2 / span 2"
            :placeholder="pclText('')"
            v-model="values.AchievementString2TextBox"
          />
        </div>
        <div
          style="
            margin: 15px 0px 10px 0px;
            display: grid;
            grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
            grid-template-rows: 35px;
          "
        >
          <PButton
            style="
              width: 140px;
              margin: 0px 15px 0px 0px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
              justify-self: end;
            "
            @click="showUnavailable"
            type="tint"
            >{{ pclText('Tools.Test.Achievement.Preview') }}</PButton
          >
          <PButton
            style="
              width: 140px;
              margin: 0px 15px 0px 0px;
              grid-row: 1 / span 1;
              grid-column: 2 / span 1;
              justify-self: start;
            "
            @click="showUnavailable"
            >{{ pclText('Tools.Test.Achievement.Save') }}</PButton
          >
        </div>
      </div>
    </PCard>
    <PCard :title="pclText('Tools.Test.Avatar.Title')" default-fold-status="fold">
      <div style="margin: 0px 25px 20px 25px; display: flex; flex-direction: column">
        <div
          style="
            margin: 0px 0px 10px 0px;
            display: grid;
            grid-template-columns: auto minmax(0, 1fr);
          "
        >
          <span
            style="
              margin: 0px 10px 0px 0px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
              align-self: center;
            "
            class="pcl-text"
            >{{ pclText('Tools.Test.Avatar.Size') }}</span
          >
          <PComboBox
            style="height: 26px; grid-row: 1 / span 1; grid-column: 2 / span 1"
            :options="
              ['64x64', '96x96', '128x128'].map((size, index) => ({
                key: String(index),
                text: size,
              }))
            "
            v-model="values.CmbHeadSize"
          />
        </div>
        <div style="display: grid; grid-template-columns: minmax(0, 1fr) auto auto minmax(0, 1fr)">
          <PButton
            style="
              width: 140px;
              height: 35px;
              margin: 0px 10px 0px 0px;
              grid-row: 1 / span 1;
              grid-column: 2 / span 1;
            "
            @click="showUnavailable"
            >{{ pclText('Tools.Test.Avatar.SelectSkin') }}</PButton
          >
          <PButton
            style="width: 140px; height: 35px; grid-row: 1 / span 1; grid-column: 3 / span 1"
            @click="showUnavailable"
            >{{ pclText('Tools.Test.Avatar.Save') }}</PButton
          >
        </div>
      </div>
    </PCard>
  </section>
</template>

<style scoped>
.skin-tool {
  margin: 2px 25px 20px;
  container-type: inline-size;
}
.skin-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 0fr);
  gap: 0;
  align-items: center;
  transition:
    grid-template-columns 350ms ease,
    gap 350ms ease;
}
.skin-layout.has-skin {
  grid-template-columns: minmax(0, 1fr) minmax(0, 0.8fr);
  gap: 24px;
}
.skin-controls {
  min-width: 0;
}
.skin-query {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 8px;
}
.skin-query :deep(.input-wrapper) {
  flex: 1;
  min-width: 0;
}
.skin-status {
  margin-top: 8px;
  color: var(--color-text-grey);
}
.skin-result-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 18px;
}
.skin-result-name {
  flex: 1 1 120px;
  min-width: 0;
  color: var(--color-text-grey);
  overflow-wrap: anywhere;
}
.skin-result-name strong {
  color: var(--color-text);
}
.skin-preview {
  min-width: 0;
  overflow: hidden;
  display: flex;
  justify-content: center;
}
.skin-preview-content {
  width: min(100%, 200px);
}
.skin-preview canvas {
  display: block;
  width: 100%;
  height: auto;
  aspect-ratio: 2 / 3;
}
.skin-reveal-enter-active,
.skin-reveal-leave-active {
  transition: opacity 250ms ease;
}
.skin-reveal-enter-active {
  transition-delay: 100ms;
}
.skin-reveal-enter-from,
.skin-reveal-leave-to {
  opacity: 0;
}
@container (max-width: 360px) {
  .skin-layout.has-skin {
    grid-template-columns: minmax(0, 1fr);
    gap: 10px;
  }
  .skin-preview-content {
    max-width: 160px;
  }
}
@media (prefers-reduced-motion: reduce) {
  .skin-layout,
  .skin-reveal-enter-active,
  .skin-reveal-leave-active {
    transition: none;
  }
}
</style>
