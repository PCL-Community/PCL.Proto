<script setup lang="ts">
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import { computed, onMounted, onUnmounted } from 'vue'
import exportOptions from '@/assets/export-options.json'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { usePageSettings } from '@/composables/usePageSettings'
import PButton from '@/components/widget/PButton.vue'
import { FloatButtonType, useFloatButton } from '@/composables/useFloatButton'
import { useTaskManager } from '@/stores/task'
import PCard from '@/components/widget/PCard.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
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

const text = pclText
</script>
<template>
  <div class="pcl-page">
    <div>
      <div data-control="PanAllBack" style="display: grid">
        <div data-control="PanBack" style="grid-row: 1 / span 1; grid-column: 1 / span 1">
          <div style="margin: 10px 25px 65px 25px; display: flex; flex-direction: column">
            <PHint
              v-if="values.CheckAdvancedModrinth && instance.plugins.includes('optifine')"
              data-control="HintOptiFine"
              style="margin: 15px 0px 0px 0px"
              severity="warning"
              >{{ text('Instance.Export.ModrinthWarning') }}</PHint
            >
            <PCard style="margin: 15px 0px" hide-title>
              <div
                style="
                  margin: 15px 25px 15px 22px;
                  display: grid;
                  grid-template-columns: auto minmax(0, 1fr) auto minmax(0, 0.3fr);
                  grid-template-rows: 28px;
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
                  >{{ text('Instance.Export.PackName') }}</span
                >
                <PTextBox
                  data-control="TextExportName"
                  style="grid-row: 1 / span 1; grid-column: 2 / span 1"
                  :placeholder="text('')"
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
                  >{{ text('Instance.Export.PackVersion') }}</span
                >
                <PTextBox
                  data-control="TextExportVersion"
                  style="grid-row: 1 / span 1; grid-column: 4 / span 1"
                  :placeholder="text('')"
                  v-model="values.TextExportVersion"
                />
              </div>
            </PCard>
            <PCard
              data-control="CardOptions"
              style="min-height: 39px; margin: 0px 0px 15px 0px"
              hide-title
            >
              <div class="export-options-heading">
                <h2>{{ text('Instance.Export.OptionListTitle') }}</h2>
                <PButton inline @click="() => reset()"
                  ><PSvgIcon name="lucide/rotate-ccw" />{{ text('Common.Action.Reset') }}</PButton
                >
              </div>
              <div
                data-control="PanOptions"
                style="padding: 0 25px 15px; display: flex; flex-direction: column"
              >
                <PCheckBox
                  v-for="option in visibleOptions"
                  :key="option.id"
                  :id="`PageInstanceExport-${option.id}`"
                  :data-control="option.id"
                  :style="{ marginLeft: option.indent ? '30px' : '0', minHeight: '26px' }"
                  :title="text(option.description)"
                  :disabled="option.disabled"
                  :label="text(option.title)"
                  :model-value="option.disabled || Boolean(values[option.id])"
                  @update:model-value="(value) => (values[option.id] = value)"
                />
              </div>
            </PCard>
            <PCard
              style="margin: 0px 0px 15px 0px"
              :title="text('Instance.Export.Advanced.Title')"
              default-fold-status="fold"
            >
              <div style="margin: 0px 23px 20px 25px; display: flex; flex-direction: column">
                <PHint style="margin: 2px 0px 8px 0px" severity="warning">{{
                  text('Instance.Export.Advanced.BundleHint')
                }}</PHint>
                <PCheckBox
                  data-control="CheckAdvancedInclude"
                  style="height: 26px"
                  :title="text('Instance.Export.Advanced.BundleFiles.ToolTip')"
                  id="PageInstanceExport-CheckAdvancedInclude"
                  :label="text('Instance.Export.Advanced.BundleFiles')"
                  v-model="values.CheckAdvancedInclude"
                />
                <PCheckBox
                  data-control="CheckAdvancedModrinth"
                  style="height: 26px"
                  :title="text('Instance.Export.Advanced.Modrinth.ToolTip')"
                  id="PageInstanceExport-CheckAdvancedModrinth"
                  :label="text('Instance.Export.Advanced.Modrinth')"
                  v-model="values.CheckAdvancedModrinth"
                />
                <PHint style="margin: 20px 0px 2px 0px" severity="info" closable>{{
                  text('Instance.Export.Advanced.ConfigHint')
                }}</PHint>
                <div
                  style="
                    height: 35px;
                    margin: 12px 0px 0px 0px;
                    display: grid;
                    grid-template-columns: auto auto auto;
                  "
                >
                  <PButton
                    data-control="BtnAdvancedImport"
                    style="
                      min-width: 140px;
                      margin: 0px 20px 0px 0px;
                      padding: 0px 13px;
                      grid-row: 1 / span 1;
                      grid-column: 1 / span 1;
                    "
                    @click="showUnavailable"
                    type="tint"
                    >{{ text('Instance.Export.Advanced.ImportConfig') }}</PButton
                  >
                  <PButton
                    data-control="BtnAdvancedExport"
                    style="
                      min-width: 140px;
                      margin: 0px 20px 0px 0px;
                      padding: 0px 13px;
                      grid-row: 1 / span 1;
                      grid-column: 2 / span 1;
                    "
                    @click="showUnavailable"
                    >{{ text('Instance.Export.Advanced.ExportConfig') }}</PButton
                  >
                </div>
              </div>
            </PCard>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.export-options-heading {
  padding: 10px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.export-options-heading h2 {
  font-size: 13px;
}
</style>
