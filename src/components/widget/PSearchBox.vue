<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import PButton from './PButton.vue'
import PCard from './PCard.vue'
import PSvgIcon from './PSvgIcon.vue'
defineProps<{ placeholder?: string; searchButton?: boolean; label?: string }>()
const emit = defineEmits(['search'])
const query = defineModel<string>({ default: '' })
</script>

<template>
  <PCard hide-title>
    <section class="search-card">
      <PSvgIcon name="lucide/search" :size="14" class="search-icon" />
      <span v-if="label" class="search-label">{{ label }}</span>
      <input
        type="text"
        :placeholder="placeholder"
        :aria-label="label || placeholder || pclText('Common.Action.Search')"
        maxlength="50"
        class="search-input"
        v-model="query"
        v-on:keydown.enter="emit('search', query)"
      />
      <!-- 清除按钮始终占位，仅按 CE 的 90 ms 淡入淡出，避免输入时布局抖动 -->
      <button
        type="button"
        class="pcl-icon-button search-clear"
        :class="{ 'is-visible': query }"
        :aria-label="pclText('Common.Action.Clear')"
        :aria-hidden="query ? undefined : 'true'"
        :tabindex="query ? undefined : -1"
        :title="pclText('Common.Action.Clear')"
        @click="
          () => {
            query = ''
            emit('search', '')
          }
        "
      >
        <PSvgIcon name="lucide/x" :size="14" />
      </button>
      <PButton
        v-if="searchButton"
        type="tint"
        class="search-button"
        @click="emit('search', query)"
        >{{ pclText('Common.Action.Search') }}</PButton
      >
    </section>
  </PCard>
</template>

<style scoped>
.search-card {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  gap: 10px;
  height: 40px;
  align-items: center;
  padding: 4px 4px 4px 14px;
}

.search-icon {
  color: var(--color-text-grey);
}

.search-label {
  flex: 0 0 auto;
  color: var(--color-text);
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
}

.search-input {
  flex: 1;
  min-width: 0;
  height: 100%;
  padding: 0;
  font-size: 13.5px;
  text-align: left;
  color: var(--color-text-black);
  font-family: inherit;
  border: none;
  outline: none;
  background-color: transparent;
}
.search-clear {
  flex-shrink: 0;
  margin-right: 6px;
  color: var(--color-text-grey);
  opacity: 0;
  pointer-events: none;
  /* CE MySearchBox：清除按钮 90 ms 线性淡入淡出。 */
  transition:
    opacity 90ms linear,
    color 150ms linear,
    transform 250ms cubic-bezier(0.34, 2, 0.64, 1);
}
.search-clear.is-visible {
  opacity: 1;
  pointer-events: auto;
}
.search-clear:not(:disabled):hover {
  color: var(--color-text-black);
}
.search-button {
  width: 60px;
  height: 32px;
  padding: 0;
  flex-shrink: 0;
}
</style>
