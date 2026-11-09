<script setup lang="ts">
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import { computed, onMounted, onUnmounted } from 'vue'
import exportOptions from '@/assets/export-options.json'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { usePageSettings } from '@/composables/usePageSettings'
import PButton from '@/components/widget/PButton.vue'
import PIconTextButton from '@/components/widget/PIconTextButton.vue'
import { FloatButtonType, useFloatButton } from '@/composables/useFloatButton'
import { useTaskManager } from '@/stores/task'
import PCard from '@/components/widget/PCard.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
const instance = useSelectedInstance()
const { floatButtonState, setFloatButton } = useFloatButton()
onMounted(() => {
  floatButtonState.action = showUnavailable
  setFloatButton(FloatButtonType.ExportInstance)
  floatButtonState.visible = true
})
onUnmounted(() => {
  if (floatButtonState.action !== showUnavailable) return
  floatButtonState.action = undefined
  const hasTasks = useTaskManager().activeTaskCount > 0
  if (hasTasks) setFloatButton(FloatButtonType.TaskManage)
  floatButtonState.visible = hasTasks
})
const defaults = {
  TextExportName: '',
  TextExportVersion: '',
  CheckAdvancedInclude: false,
  CheckAdvancedModrinth: false,
  ...Object.fromEntries(exportOptions.map((option) => [option.id, option.checked])),
}
const { values, reset } = usePageSettings('PageInstanceExport', defaults)
const visibleOptions = computed(() =>
  exportOptions.filter(
    (option) =>
      (!option.parent || values[option.parent]) &&
      (!option.require ||
        !instance.instance_info ||
        (option.require === 'optifine'
          ? instance.plugins.includes('optifine')
          : instance.plugins.some((plugin) =>
              [
                'forge',
                'neoforge',
                'fabric',
                'legacyfabric',
                'quilt',
                'liteloader',
                'cleanroom',
              ].includes(plugin),
            ))),
  ),
)
</script>
<template>
  <div class="pcl-page">
    <PHint
      v-if="values.CheckAdvancedModrinth && instance.plugins.includes('optifine')"
      severity="warning"
      >{{ pclText('Instance.Export.ModrinthWarning') }}</PHint
    >
    <PCard hide-title>
      <div
        style="
          margin: 15px 25px 15px 22px;
          display: grid;
          grid-template-columns: auto minmax(0, 1fr) auto minmax(0, 0.3fr);
        "
      >
        <span
          style="
            grid-row: 1 / span 1;
            grid-column: 1 / span 1;
            justify-self: start;
            align-self: center;
          "
          class="pcl-text pcl-grid-label"
          >{{ pclText('Instance.Export.PackName') }}</span
        >
        <PTextBox
          style="grid-row: 1 / span 1; grid-column: 2 / span 1"
          :placeholder="pclText('')"
          v-model="values.TextExportName"
        />
        <span
          style="
            margin: 0px 20px 0px 30px;
            grid-row: 1 / span 1;
            grid-column: 3 / span 1;
            justify-self: start;
            align-self: center;
          "
          class="pcl-text"
          >{{ pclText('Instance.Export.PackVersion') }}</span
        >
        <PTextBox
          style="grid-row: 1 / span 1; grid-column: 4 / span 1"
          :placeholder="pclText('')"
          v-model="values.TextExportVersion"
        />
      </div>
    </PCard>
    <PCard :title="pclText('Instance.Export.OptionListTitle')">
      <template #header-actions>
        <PIconTextButton icon="lucide/rotate-ccw" @click="() => reset()">{{
          pclText('Common.Action.Reset')
        }}</PIconTextButton>
      </template>
      <div class="pcl-card-body">
        <PCheckBox
          v-for="option in visibleOptions"
          :key="option.id"
          :id="`PageInstanceExport-${option.id}`"
          :style="{ marginLeft: option.indent ? '30px' : '0' }"
          :title="pclText(option.description)"
          :disabled="option.disabled"
          :label="pclText(option.title)"
          :model-value="option.disabled || Boolean(values[option.id])"
          @update:model-value="(value) => (values[option.id] = value)"
        />
      </div>
    </PCard>
    <PCard :title="pclText('Instance.Export.Advanced.Title')" default-fold-status="fold">
      <div class="pcl-card-body">
        <PHint style="margin: 2px 0px 8px 0px" severity="warning">{{
          pclText('Instance.Export.Advanced.BundleHint')
        }}</PHint>
        <PCheckBox
          :title="pclText('Instance.Export.Advanced.BundleFiles.ToolTip')"
          id="PageInstanceExport-CheckAdvancedInclude"
          :label="pclText('Instance.Export.Advanced.BundleFiles')"
          v-model="values.CheckAdvancedInclude"
        />
        <PCheckBox
          :title="pclText('Instance.Export.Advanced.Modrinth.ToolTip')"
          id="PageInstanceExport-CheckAdvancedModrinth"
          :label="pclText('Instance.Export.Advanced.Modrinth')"
          v-model="values.CheckAdvancedModrinth"
        />
        <PHint style="margin: 20px 0px 2px 0px" severity="info" closable>{{
          pclText('Instance.Export.Advanced.ConfigHint')
        }}</PHint>
        <div class="pcl-button-row pcl-button-row-fill pcl-button-row-spaced">
          <PButton
            style="min-width: 140px; padding: 0px 13px"
            @click="showUnavailable"
            type="tint"
            >{{ pclText('Instance.Export.Advanced.ImportConfig') }}</PButton
          >
          <PButton style="min-width: 140px; padding: 0px 13px" @click="showUnavailable">{{
            pclText('Instance.Export.Advanced.ExportConfig')
          }}</PButton>
        </div>
      </div>
    </PCard>
  </div>
</template>
