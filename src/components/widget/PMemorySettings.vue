<script setup lang="ts">
import { toRef } from 'vue'
import { pclText as text } from '@/api/pclLocale'
import { memoryTickToMib } from '@/api/memorySettings'
import { memoryGiB, useMemorySettings } from '@/composables/useMemorySettings'
import PErrorRetry from './PErrorRetry.vue'

const props = defineProps<{ instance?: boolean; directory?: string; java64: boolean | null }>()
const { mode, maximum, initial, ready, error, preview, segments, load, save } = useMemorySettings(
  props.instance ? toRef(props, 'directory') : undefined,
  toRef(props, 'java64'),
)
</script>

<template>
  <div class="memory-settings">
    <div class="memory-options">
      <label v-if="instance" data-control="RadioRamType2" class="pcl-radio inherit">
        <input v-model="mode" :disabled="!ready" type="radio" value="inherit" @change="save" />{{
          text('Instance.Setup.FollowGlobal')
        }}
      </label>
      <label
        data-control="RadioRamType0"
        class="pcl-radio"
        :title="text('Setup.Launch.Memory.Auto.ToolTip')"
      >
        <input v-model="mode" :disabled="!ready" type="radio" value="auto" @change="save" />{{
          text('Setup.Launch.Memory.Auto')
        }}
      </label>
      <label data-control="RadioRamType1" class="pcl-radio">
        <input v-model="mode" :disabled="!ready" type="radio" value="custom" @change="save" />{{
          text('Common.Option.Customize')
        }}
      </label>
      <span class="pcl-text" :title="text('Setup.Launch.Memory.Max.ToolTip')"
        >{{ text('Setup.Launch.Memory.Max')
        }}<template v-if="mode === 'custom'"> ({{ memoryTickToMib(maximum) }} MiB)</template></span
      >
      <input
        v-model.number="maximum"
        data-control="SliderRamCustom"
        :disabled="!ready || mode !== 'custom'"
        type="range"
        min="0"
        max="49"
        @change="save"
      />
      <span class="pcl-text" :title="text('Setup.Launch.Memory.Initial.ToolTip')"
        >{{ text('Setup.Launch.Memory.Initial')
        }}<template v-if="mode === 'custom'">
          ({{
            initial
              ? `${Math.min(memoryTickToMib(initial), memoryTickToMib(maximum))} MiB`
              : text('Setup.Launch.Memory.Auto')
          }})</template
        ></span
      >
      <input
        v-model.number="initial"
        data-control="SliderRamInitialCustom"
        :disabled="!ready || mode !== 'custom'"
        :title="text('Setup.Launch.Memory.Initial.ToolTip')"
        type="range"
        min="0"
        max="49"
        @change="save"
      />
    </div>
    <PErrorRetry :error="error" @retry="ready ? save() : load()" />
    <div data-control="PanRamDisplay" class="memory-display">
      <div
        data-control="RectRamUsed"
        class="pcl-divider bar"
        :style="{ width: `${segments.used}%` }"
      />
      <div
        data-control="RectRamGame"
        class="pcl-divider bar game"
        :style="{ width: `${segments.game}%`, marginLeft: `${segments.used}%` }"
      />
      <div
        data-control="RectRamEmpty"
        class="pcl-divider bar empty"
        :style="{ width: `${segments.empty}%`, marginLeft: `${segments.used + segments.game}%` }"
      />
      <span data-control="LabRamUsedTitle" class="memory-title used-title">{{
        text(instance ? 'Instance.Setup.Memory.Used' : 'Setup.Launch.Memory.UsedTotal')
      }}</span>
      <span data-control="LabRamGameTitle" class="memory-title game-title">{{
        text('Proto.Settings.MemoryEstimate')
      }}</span>
      <div class="memory-used">
        <span data-control="LabRamUsed">{{
          preview ? memoryGiB(preview.total_bytes - preview.available_bytes) : '—'
        }}</span
        ><span data-control="LabRamTotal">{{
          preview ? ` / ${memoryGiB(preview.total_bytes)}` : ' / —'
        }}</span>
      </div>
      <span data-control="LabRamGame" class="memory-game">{{
        preview?.allocation_mib != null ? memoryGiB(preview.allocation_mib * 1048576) : '—'
      }}</span>
    </div>
  </div>
</template>

<style scoped>
.memory-settings {
  margin: 0 25px 15px;
  display: flex;
  flex-direction: column;
}
.memory-options {
  margin-bottom: 3px;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  row-gap: 9px;
  align-items: center;
}
.memory-options label {
  grid-column: 1 / -1;
  justify-self: start;
  margin-right: 20px;
}
.memory-options span {
  margin-right: 20px;
}
.memory-options input[type='range'] {
  margin-right: 12px;
}
.memory-display {
  margin: 11px 2px 0 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  grid-template-rows: minmax(0, 1fr) 4px minmax(0, 1fr);
}
.bar {
  height: 4px;
  grid-row: 2;
  grid-column: 1 / span 2;
  justify-self: start;
}
.game {
  opacity: 0.5;
}
.empty {
  opacity: 0.7;
}
.memory-title {
  margin: 0 0 4px 2px;
  font-size: 11px;
  opacity: 0.7;
  grid-row: 1;
}
.used-title {
  grid-column: 1;
  justify-self: start;
}
.game-title {
  grid-column: 2;
  justify-self: end;
}
.memory-used {
  margin: 3px 0 0 2px;
  grid-row: 3;
  grid-column: 1;
  justify-self: start;
  display: flex;
  flex-wrap: wrap;
  min-width: 0;
  align-items: center;
  font-size: 16px;
}
.memory-game {
  margin: 3px 0 0 2px;
  font-size: 16px;
  grid-row: 3;
  grid-column: 2;
  justify-self: end;
}
</style>
