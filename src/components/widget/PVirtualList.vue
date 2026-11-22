<script setup lang="ts" generic="T">
import { computed, onMounted, onScopeDispose, ref, useTemplateRef, watch } from 'vue'

const props = withDefaults(
  defineProps<{ items: readonly T[]; itemKey: (item: T) => string; rowHeight?: number }>(),
  { rowHeight: 42 },
)
defineSlots<{ default(props: { item: T }): unknown }>()
const root = useTemplateRef<HTMLElement>('root')
// shortcut: each list has a fixed row height; use measured heights for multiline rows.
const start = ref(0)
const end = ref(16)
const visible = computed(() => props.items.slice(start.value, end.value))
let scroller: HTMLElement | null = null
let observer: ResizeObserver | undefined
let frame = 0

function findScroller(node: HTMLElement | null) {
  let current = node?.parentElement
  while (current && current !== document.body) {
    const overflowY = getComputedStyle(current).overflowY
    if (overflowY === 'auto' || overflowY === 'scroll' || overflowY === 'overlay') return current
    current = current.parentElement
  }
  return document.scrollingElement as HTMLElement | null
}

function measure() {
  frame = 0
  if (!root.value || !scroller) return
  const list = root.value.getBoundingClientRect()
  const view = scroller.getBoundingClientRect()
  start.value = Math.min(
    props.items.length,
    Math.max(0, Math.floor((view.top - list.top) / props.rowHeight) - 4),
  )
  end.value = Math.min(
    props.items.length,
    Math.max(start.value, Math.ceil((view.bottom - list.top) / props.rowHeight) + 4),
  )
}
function schedule() {
  if (!frame) frame = requestAnimationFrame(measure)
}
watch(() => [props.items, props.rowHeight], schedule, { flush: 'post' })
onMounted(() => {
  scroller = findScroller(root.value)
  scroller?.addEventListener('scroll', schedule, { passive: true })
  observer = new ResizeObserver(schedule)
  if (scroller) observer.observe(scroller)
  const card = root.value!.closest('.mycard-container')
  if (card) observer.observe(card)
  observer.observe(root.value!)
  measure()
})
onScopeDispose(() => {
  scroller?.removeEventListener('scroll', schedule)
  observer?.disconnect()
  if (frame) cancelAnimationFrame(frame)
})
</script>

<template>
  <div
    ref="root"
    class="virtual-list"
    role="list"
    :style="{ height: `${items.length * rowHeight}px`, '--virtual-row-height': `${rowHeight}px` }"
  >
    <div class="virtual-items" :style="{ top: `${start * rowHeight}px` }">
      <div
        v-for="(item, index) in visible"
        :key="itemKey(item)"
        class="virtual-row"
        role="listitem"
        :aria-posinset="start + index + 1"
        :aria-setsize="items.length"
      >
        <slot :item="item" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.virtual-list {
  position: relative;
  flex-shrink: 0;
}
.virtual-items {
  position: absolute;
  left: 0;
  right: 0;
}
.virtual-row {
  height: var(--virtual-row-height);
}
.virtual-row :deep(.gameinfo-container) {
  height: var(--virtual-row-height);
  box-sizing: border-box;
}
.virtual-row :deep(.title),
.virtual-row :deep(.subtitle) {
  display: block;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
</style>
