<script setup lang="ts">
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import PCard from '@/components/widget/PCard.vue'
import PListItem from '@/components/widget/PListItem.vue'
import PHint from '@/components/widget/PHint.vue'
import PIconTextButton from '@/components/widget/PIconTextButton.vue'
import { javaDisplayName, type IJavaRuntimeInfo } from '@/api/javaReq'
import { onMounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import sideTip from '@/composables/sideTip'
import { pclText } from '@/api/pclLocale'
import { useJavaStore } from '@/stores/java'
import { useModal } from '@/composables/useModal'
const java = useJavaStore()
const modal = useModal()
const loading = ref(false)
const saving = ref(false)
const error = ref<string | null>(null)
const architectureLabels = {
  X86: 'x86',
  X64: 'x86-64',
  Arm64: 'ARM64',
  FatFile: 'Universal',
  Unknown: 'Unknown',
}
async function load(refresh = false) {
  if (!isTauri()) {
    error.value = pclText('Proto.Desktop.JavaScan')
    return
  }
  loading.value = true
  error.value = null
  try {
    await java.load(undefined, refresh)
  } catch (err) {
    error.value = pclText('Proto.Java.ReadFailed')
    console.error(err)
  } finally {
    loading.value = false
  }
}
async function change(operation: () => Promise<unknown>) {
  if (!isTauri()) {
    sideTip.show(pclText('Proto.Desktop.JavaScan'), 'warn')
    return
  }
  if (saving.value || loading.value) return
  saving.value = true
  try {
    await operation()
  } catch (err) {
    error.value = String(err)
    sideTip.show(String(err), 'warn')
  } finally {
    saving.value = false
  }
}
async function select(runtime: IJavaRuntimeInfo | null) {
  if (runtime && !runtime.is_enabled) {
    sideTip.show(pclText('Setup.Java.EnableBeforeSelect'), 'warn')
    return
  }
  await change(() => java.selectGlobal(runtime?.java_exe || null))
}
async function toggle(runtime: IJavaRuntimeInfo) {
  if (runtime.is_enabled && java.globalJava === runtime.java_exe) {
    sideTip.show(pclText('Setup.Java.DeselectBeforeDisable'), 'warn')
    return
  }
  await change(() => java.setEnabled(runtime))
}
async function addJava() {
  await change(async () => {
    const runtime = await invoke<IJavaRuntimeInfo | null>('add_java')
    if (runtime) {
      await java.load()
      sideTip.show(pclText('Setup.Java.Added'), 'success')
    }
  })
}
function info(runtime: IJavaRuntimeInfo) {
  void modal.open({
    title: pclText('Setup.Java.Info.Title'),
    content: pclText('Setup.Java.Info.Format', [
      runtime.is_jre ? 'JRE' : 'JDK',
      runtime.version,
      architectureLabels[runtime.architecture],
      runtime.is_64_bit ? '64 Bit' : '32 Bit',
      runtime.vendor || pclText('Common.State.Unknown'),
      runtime.java_folder,
    ]),
    buttons: [{ content: pclText('Common.Action.Close'), operation: () => modal.close() }],
  })
}
onMounted(() => load())
</script>
<template>
  <section class="pcl-page">
    <PCard hide-title>
      <div style="margin: 8px; display: flex; gap: 8px">
        <PIconTextButton icon="lucide/circle-plus" @click="addJava" :disabled="loading || saving">
          {{ pclText('Setup.Java.Add') }}
        </PIconTextButton>
        <PIconTextButton
          icon="lucide/refresh-cw"
          @click="() => load(true)"
          :disabled="loading || saving"
        >
          {{ pclText('Common.Action.Refresh') }}
        </PIconTextButton>
      </div>
    </PCard>
    <PCard hide-title>
      <div style="margin: 15px; display: flex; flex-direction: column">
        <PHint v-if="error" severity="error">{{ error }}</PHint>
        <PHint v-if="loading" severity="info">{{ pclText('Setup.Java.Loading') }}</PHint>
        <PListItem
          :title="pclText('Setup.Java.AutoSelect.Title')"
          :subtitle="pclText('Setup.Java.AutoSelect.Info')"
          clickable
          :selected="!java.globalJava"
          @click="select(null)"
        >
        </PListItem>
        <PListItem
          v-for="runtime in java.runtimes"
          :key="runtime.java_exe"
          :class="{ 'java-disabled': !runtime.is_enabled }"
          :title="javaDisplayName(runtime)"
          :tags="[architectureLabels[runtime.architecture], runtime.vendor].filter(Boolean)"
          :subtitle="runtime.java_folder"
          clickable
          :selected="java.globalJava === runtime.java_exe"
          @click="select(runtime)"
        >
          <template #icon-actions>
            <button
              class="pcl-icon-button"
              type="button"
              :title="pclText('Common.Action.Open')"
              :aria-label="pclText('Common.Action.Open')"
              @click="() => change(() => invoke('open_java_folder', { javaExe: runtime.java_exe }))"
              :disabled="saving || loading"
            >
              <PSvgIcon name="lucide/folder-open" :size="15" />
            </button>
            <button
              class="pcl-icon-button"
              type="button"
              :title="pclText('Setup.Java.Detail.ToolTip')"
              :aria-label="pclText('Setup.Java.Detail.ToolTip')"
              @click="info(runtime)"
            >
              <PSvgIcon name="lucide/info" :size="15" />
            </button>
            <button
              class="pcl-icon-button"
              type="button"
              :title="pclText(runtime.is_enabled ? 'Setup.Java.Disable' : 'Setup.Java.Enable')"
              :aria-label="pclText(runtime.is_enabled ? 'Setup.Java.Disable' : 'Setup.Java.Enable')"
              @click="toggle(runtime)"
              :disabled="saving || loading"
            >
              <PSvgIcon
                :name="runtime.is_enabled ? 'lucide/circle-minus' : 'lucide/circle-check'"
                :size="15"
              />
            </button>
          </template>
        </PListItem>
        <PHint v-if="!loading && !java.runtimes.length && !error" severity="info">{{
          pclText('Instance.Setup.Options.Java.NoRuntime')
        }}</PHint>
      </div>
    </PCard>
  </section>
</template>
<style scoped>
.java-disabled {
  opacity: 0.55;
}
.java-disabled :deep(.title) {
  text-decoration: line-through;
}
</style>
