<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { pclText } from '@/api/pclLocale'
import PCard from './PCard.vue'

const props = defineProps<{ count: number }>()
const toolbar = ref<HTMLElement>()
let animation: Animation | undefined
watch(
  () => props.count,
  async (count, previous) => {
    if (!count || count <= (previous || 0)) return
    await nextTick()
    if (!toolbar.value || window.matchMedia('(prefers-reduced-motion: reduce)').matches) return
    animation?.cancel()
    animation = toolbar.value.animate(
      [
        { transform: `translateY(${previous ? 0 : 15}px)`, opacity: previous ? 1 : 0, offset: 0 },
        { transform: 'translateY(-2px)', opacity: 1, offset: 1 / 3 },
        { transform: 'translateY(1px)', opacity: 1, offset: 3 / 4 },
        { transform: 'translateY(0)', opacity: 1, offset: 1 },
      ],
      { duration: 360, easing: 'ease-in-out' },
    )
  },
  { immediate: true },
)
</script>

<template>
  <div v-if="count" ref="toolbar" class="pcl-selection-toolbar">
    <PCard hide-title>
      <p>{{ pclText('Instance.Resource.SelectedCount', [count]) }}</p>
      <div><slot /></div>
    </PCard>
  </div>
</template>
