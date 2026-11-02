<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import { onMounted, onUnmounted, useTemplateRef, ref, provide } from 'vue'
import useSideNavState from '@/stores/windowState'
import SideNavItem from '@/components/widget/SideNavItem.vue'
import { type INavItemGroup } from '@/types/naviOptions'
import { useModal } from '@/composables/useModal'

defineProps<{ sideNavGroups: INavItemGroup[] }>()

const revision = ref(0)
provide('navAction', (kind: string) => {
  if (kind === 'refresh') revision.value++
  else
    void useModal().open({
      title: pclText('Common.Dialog.Title'),
      content: pclText('Proto.NotImplemented.Initialize'),
    })
})
let observer: ResizeObserver | null = null
const asideRef = useTemplateRef<HTMLElement>('asideRef')
const sideNavState = useSideNavState()

function updateAsideBackgroundWidth() {
  if (asideRef.value) {
    sideNavState.setWidth(asideRef.value.offsetWidth)
  }
}

onMounted(() => {
  asideRef.value?.querySelectorAll('.sidenav-line').forEach((el_, i) => {
    let el = el_ as HTMLDivElement
    el.style.animationPlayState = 'paused'
    el.style.animationDelay = `${i * 0.02}s`
    requestAnimationFrame(() => {
      el.style.animationPlayState = 'running'
    })
  })
  observer = new ResizeObserver(updateAsideBackgroundWidth)
  observer.observe(asideRef.value!)
})

onUnmounted(() => {
  observer?.disconnect()
})
</script>

<template>
  <div class="view-content">
    <aside ref="asideRef">
      <div v-for="(group, index) in sideNavGroups" :key="group.title || index">
        <p v-if="group.title" class="side-group-title sidenav-line">{{ group.title }}</p>
        <div class="side-group-content">
          <SideNavItem
            v-for="item in group.content"
            :key="item.linkto || item.text"
            v-bind="item"
          />
        </div>
      </div>
    </aside>
    <article class="subview">
      <RouterView :key="revision" v-slot="{ Component }">
        <component :is="Component" />
      </RouterView>
    </article>
  </div>
</template>

<style scoped>
.side-group-content {
  display: flex;
  flex-direction: column;
}

.side-group-title {
  margin: 6px 5px 4px 13px;
  font-size: 12px;
  color: var(--color-text-grey);
}

article {
  flex: 1 1 auto;
  min-width: 0;
  overflow-y: auto;
}

aside {
  padding: 12px 0 0;
  width: max-content;
  white-space: nowrap;
  overflow-x: hidden;
  overflow-y: auto;
  height: 100%;
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

@keyframes fade-in-left {
  70% {
    transform: translateX(2px);
  }
  100% {
    opacity: 1;
    transform: translateX(0);
  }
}

aside :deep(.sidenav-line) {
  opacity: 0;
  will-change: transform, opacity;
  transform: translateX(-100%);
  animation: fade-in-left 0.45s ease-in-out forwards;
}
</style>
