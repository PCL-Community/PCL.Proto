<script setup lang="ts">
import { sideNavState } from '@/windowState';
import { onMounted, ref } from 'vue';
import SideGroup from '@/components/SideGroup.vue'
import { downloadSubViewManifest } from '@/options/naviOptions';

let observer: ResizeObserver | null = null

const asideRef = ref<HTMLElement>()

function updateAsideBackgroundWidth() {
  if (asideRef.value) {
    sideNavState.width = asideRef.value.offsetWidth
  }
}

onMounted(() => {
  observer = new ResizeObserver(updateAsideBackgroundWidth)
  if (asideRef.value) { observer.observe(asideRef.value) }
})
</script>

<template>
  <div class="view-content">
    <aside ref="asideRef">
      <SideGroup v-for="group in downloadSubViewManifest" :title="group.title" :content="group.content" />
    </aside>
    <article>
      <RouterView />
    </article>
  </div>
</template>

<style scoped>
i.refresh-icon {
  margin-left: auto;
  color: var(--light-blue);
  opacity: 0;
  transition: 0.4s;
  margin-right: 10px;
  /* visibility: hidden; */
  display: flex;
  align-items: center;
  justify-content: center;
}

i.refresh-icon:hover {
  color: var(--color-titlebar);
}

article {
  flex: 1 1 0;
  overflow: auto;
}

.view-content {
  display: flex;
  width: 100%;
  height: 100%;
  box-sizing: border-box;
}

aside {
  padding: 14px 0 0 0;
  height: 100%;
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  gap: 28px;
}
</style>