<script setup lang="ts">
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import { ref, defineAsyncComponent, watch } from 'vue'
import { useRouter } from 'vue-router'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { basename, dirname, join } from '@tauri-apps/api/path'
import { open, save } from '@tauri-apps/plugin-dialog'
import { useTaskManager } from '@/stores/task'
import { usePageSettings } from '@/composables/usePageSettings'
import { useModal } from '@/composables/useModal'
import sideTip from '@/composables/sideTip'
import getSkinUrl from '@/api/skinGetter'
import MinecraftServerCard from '@/components/widget/MinecraftServerCard.vue'
const SkinViewer = defineAsyncComponent(() => import('@/components/widget/SkinViewer.vue'))
const skin = ref('')
const router = useRouter()
import PTextBox from '@/components/widget/PTextBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PCard from '@/components/widget/PCard.vue'
import PButton from '@/components/widget/PButton.vue'
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
      title: text('Tools.Test.Luck.Title'),
      content: pclText('Proto.Tools.Luck', [luck]),
      buttons: [{ type: 'tint', content: pclText('Common.Action.Confirm') }],
    })
  } catch (err) {
    sideTip.show(pclText('Proto.Tools.LuckFailed', [String(err)]), 'warn')
  }
}
async function saveSkin() {
  const name = String(values.TextSkinID || '').trim()
  if (!/^[A-Za-z0-9_]{3,16}$/.test(name)) {
    sideTip.show(pclText('Proto.Tools.InvalidPlayer'), 'warn')
    return
  }
  try {
    await getSkinUrl(
      name,
      (url) => {
        skin.value = url
        const link = document.createElement('a')
        link.href = url
        link.download = `${name}.png`
        link.click()
      },
      'username',
    )
  } catch (err) {
    sideTip.show(pclText('Proto.Tools.SkinFailed', [String(err)]), 'warn')
  }
}

const text = pclText
watch(() => values.TextDownloadUrl, (value) => {
  if (values.TextDownloadName || !value) return
  try {
    const name = new URL(String(value)).pathname.split('/').pop()
    if (name) values.TextDownloadName = decodeURIComponent(name)
  } catch { /* Wait for a complete URL. */ }
})

async function chooseDownloadFolder() {
  if (!isTauri()) return sideTip.show(pclText('Proto.Desktop.Download'), 'warn')
  try {
    const folder = await open({ directory: true, defaultPath: String(values.TextDownloadFolder || '') || undefined })
    if (folder) values.TextDownloadFolder = folder
  } catch (error) { sideTip.show(pclText('Proto.Error.Download', [String(error)]), 'warn') }
}

async function startCustomDownload() {
  if (!isTauri()) return sideTip.show(pclText('Proto.Desktop.Download'), 'warn')
  try {
    const url = new URL(String(values.TextDownloadUrl || '').trim())
    if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password) throw new Error('Invalid download URL')
    let name = String(values.TextDownloadName || '').trim()
    if (!name || /[\\/:*?"<>|\x00-\x1f]/.test(name) || name === '.' || name === '..') throw new Error('Invalid filename')
    const folder = String(values.TextDownloadFolder || '').trim()
    const destination = folder ? await join(folder, name) : await save({ defaultPath: name, title: text('Tools.Test.CustomDownload.SelectLocation') })
    if (!destination) return
    if (!folder) {
      values.TextDownloadFolder = await dirname(destination)
      name = await basename(destination)
    }
    const task = useTaskManager().downloadCustom(name, url.href, destination)
    values.TextDownloadUrl = ''
    values.TextDownloadName = ''
    sideTip.show(pclText('Download.Comp.QuickDownload.Hint.DownloadStarted', [name]), 'success')
    void task.then(() => sideTip.show(text('Tools.Test.CustomDownload.Finished', [name]), 'success'))
      .catch((error) => sideTip.show(pclText('Proto.Error.Download', [String(error)]), 'warn'))
  } catch (error) { sideTip.show(pclText('Proto.Error.Download', [String(error)]), 'warn') }
}

async function openDownloadFolder() {
  if (!isTauri()) return sideTip.show(pclText('Proto.Desktop.Download'), 'warn')
  try {
    await invoke('open_custom_download_folder', { directory: String(values.TextDownloadFolder || '').trim() })
  } catch (error) { sideTip.show(pclText('Proto.Error.Download', [String(error)]), 'warn') }
}
</script>
<template>
  <section class="pcl-page">
    <div>
      <div data-control="PanBack">
        <div
          data-control="PanMain"
          style="margin: 10px 25px; display: flex; flex-direction: column"
        >
          <PCard style="margin: 15px 0px" :title="text('Tools.Test.Title')">
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
                data-control="BtnClear"
                style="min-width: 120px; margin: 0px 15px 15px 0px"
                :title="text('Tools.Test.Clean.ToolTip')"
                @click="showUnavailable"
                >{{ text('Tools.Test.Clean.Title') }}</PButton
              >
              <PButton style="min-width: 100px; margin: 0px 15px 15px 0px" @click="showLuck">{{
                text('Tools.Test.Luck.Title')
              }}</PButton>
              <PButton
                style="min-width: 120px; margin: 0px 15px 15px 0px"
                :title="text('Tools.Test.Shortcut.ToolTip')"
                @click="showUnavailable"
                >{{ text('Tools.Test.Shortcut.Title') }}</PButton
              >
              <PButton
                style="min-width: 120px; margin: 0px 15px 15px 0px"
                @click="showUnavailable"
                >{{ text('Tools.Test.LaunchCount.Title') }}</PButton
              >
            </div>
          </PCard>
          <PCard style="margin: 0px 0px 15px 0px" :title="text('Tools.Test.CustomDownload.Title')">
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
                  >{{ text('Tools.Test.CustomDownload.Url') }}</span
                >
                <PTextBox
                  data-control="TextDownloadUrl"
                  :aria-label="text('Tools.Test.CustomDownload.Url')"
                  style="grid-row: 1 / span 1; grid-column: 2 / span 2"
                  :placeholder="text('')"
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
                  >{{ text('Tools.Test.CustomDownload.SaveTo') }}</span
                >
                <PTextBox
                  data-control="TextDownloadFolder"
                  :aria-label="text('Tools.Test.CustomDownload.SaveTo')"
                  style="margin: 10px 0px; grid-row: 3 / span 1; grid-column: 2 / span 1"
                  :placeholder="text('')"
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
                  >{{ text('Tools.Test.CustomDownload.Select') }}</PButton
                >
                <span
                  style="
                    margin: 0px 25px 0px 0px;
                    grid-row: 4 / span 1;
                    grid-column: 1 / span 1;
                    align-self: center;
                  "
                  class="pcl-text"
                  >{{ text('Tools.Test.CustomDownload.FileName') }}</span
                >
                <PTextBox
                  data-control="TextDownloadName"
                  :aria-label="text('Tools.Test.CustomDownload.FileName')"
                  style="grid-row: 4 / span 1; grid-column: 2 / span 2"
                  :placeholder="text('')"
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
                  data-control="BtnDownloadStart"
                  style="
                    width: 140px;
                    margin: 0px 15px 0px 0px;
                    grid-row: 1 / span 1;
                    grid-column: 1 / span 1;
                    justify-self: end;
                  "
                  @click="startCustomDownload"
                  type="tint"
                  >{{ text('Tools.Test.CustomDownload.Start') }}</PButton
                >
                <PButton
                  data-control="BtnDownloadOpen"
                  style="
                    width: 140px;
                    margin: 0px 15px 0px 0px;
                    grid-row: 1 / span 1;
                    grid-column: 2 / span 1;
                    justify-self: start;
                  "
                  @click="openDownloadFolder"
                  >{{ text('Common.Action.OpenFolder') }}</PButton
                >
              </div>
            </div>
          </PCard>
          <PCard
            data-control="CardSkin"
            style="margin: 0px 0px 15px 0px"
            :title="text('Tools.Test.Skin.Title')"
          >
            <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
              <div
                style="
                  margin: 14px 0px 2px 0px;
                  display: grid;
                  grid-template-columns: auto minmax(0, 1fr);
                  grid-template-rows: auto auto;
                "
              >
                <div class="pcl-form-row">
                  <label>{{ text('Tools.Test.Skin.PlayerName') }}</label
                  ><PTextBox
                    :model-value="String(values.TextSkinID || '')"
                    @update:model-value="(value) => (values.TextSkinID = value || '')"
                  />
                </div>
                <PButton
                  data-control="BtnSkinSave"
                  style="
                    min-width: 140px;
                    justify-self: start;
                    grid-row: 2;
                    grid-column: 1 / -1;
                    margin-top: 14px;
                  "
                  @click="saveSkin"
                  >{{ text('Tools.Test.Skin.Save') }}</PButton
                ><SkinViewer v-if="skin" :skin-url="skin" />
              </div>
            </div>
          </PCard>
          <PCard style="margin: 0px 0px 15px 0px" :title="text('Tools.Test.ServerQuery.Title')">
            <div style="margin: 0px 15px 15px 15px"><MinecraftServerCard embedded /></div>
          </PCard>
          <PCard
            style="margin: 0px 0px 15px 0px"
            :title="text('Tools.Test.Achievement.Title')"
            default-fold-status="fold"
          >
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
                  >{{ text('Tools.Test.Achievement.ItemId') }}</span
                >
                <PTextBox
                  data-control="AchievementBlockTextBox"
                  style="margin: 10px 0px; grid-row: 1 / span 1; grid-column: 2 / span 2"
                  :placeholder="text('')"
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
                  >{{ text('Tools.Test.Achievement.Name') }}</span
                >
                <PTextBox
                  data-control="AchievementTitleTextBox"
                  style="grid-row: 2 / span 1; grid-column: 2 / span 2"
                  :placeholder="text('')"
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
                  >{{ text('Tools.Test.Achievement.Line1') }}</span
                >
                <PTextBox
                  data-control="AchievementString1TextBox"
                  style="margin: 10px 0px; grid-row: 3 / span 1; grid-column: 2 / span 2"
                  :placeholder="text('')"
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
                  >{{ text('Tools.Test.Achievement.Line2') }}</span
                >
                <PTextBox
                  data-control="AchievementString2TextBox"
                  style="grid-row: 4 / span 1; grid-column: 2 / span 2"
                  :placeholder="text('')"
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
                  data-control="BtnAchievementPreview"
                  style="
                    width: 140px;
                    margin: 0px 15px 0px 0px;
                    grid-row: 1 / span 1;
                    grid-column: 1 / span 1;
                    justify-self: end;
                  "
                  @click="showUnavailable"
                  type="tint"
                  >{{ text('Tools.Test.Achievement.Preview') }}</PButton
                >
                <PButton
                  data-control="BtnAchievementSave"
                  style="
                    width: 140px;
                    margin: 0px 15px 0px 0px;
                    grid-row: 1 / span 1;
                    grid-column: 2 / span 1;
                    justify-self: start;
                  "
                  @click="showUnavailable"
                  >{{ text('Tools.Test.Achievement.Save') }}</PButton
                >
              </div>
            </div>
          </PCard>
          <PCard
            style="margin: 0px 0px 15px 0px"
            :title="text('Tools.Test.Avatar.Title')"
            default-fold-status="fold"
          >
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
                  >{{ text('Tools.Test.Avatar.Size') }}</span
                >
                <PComboBox
                  data-control="CmbHeadSize"
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
              <div
                style="
                  display: grid;
                  grid-template-columns: minmax(0, 1fr) auto auto minmax(0, 1fr);
                "
              >
                <PButton
                  data-control="BtnSelectSkin"
                  style="
                    width: 140px;
                    height: 35px;
                    margin: 0px 10px 0px 0px;
                    grid-row: 1 / span 1;
                    grid-column: 2 / span 1;
                  "
                  @click="showUnavailable"
                  >{{ text('Tools.Test.Avatar.SelectSkin') }}</PButton
                >
                <PButton
                  style="width: 140px; height: 35px; grid-row: 1 / span 1; grid-column: 3 / span 1"
                  @click="showUnavailable"
                  >{{ text('Tools.Test.Avatar.Save') }}</PButton
                >
              </div>
            </div>
          </PCard>
        </div>
      </div>
    </div>
  </section>
</template>
