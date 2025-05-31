<script setup lang="ts">
import { sideNavState } from '@/windowState';
import { onMounted, ref } from 'vue';
import SideGroup from '@/components/SideGroup.vue'

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
      <SideGroup />
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

/* .side-group a:hover i.refresh-icon {
  opacity: 1;
}

.side-group {
  display: flex;
  flex-direction: column;
}

.side-nav-icon {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.side-group a {
  width: 100%;
  display: flex;
  gap: 11px;
  align-items: center;
  justify-content: flex-start;
  font-size: 13px;
  color: var(--color-text-black);
  height: 34px;
  background-color: transparent;
  transition: 0.4s;
}

.side-group a:hover {
  background-color: var(--color-tint-lighter);
}

.side-group a.router-link-active {
  color: var(--color-selected);
}

.side-group a .indicator {
  visibility: hidden;
}

.side-group a.router-link-active .indicator {
  visibility: visible;
} */

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
  /* background: rgba(255, 255, 255, 0.2); */
  /* box-shadow: 0px 0px 5px rgba(0, 0, 0, 0.15); */
  padding: 12px 0 0 0;
  height: 100%;
  flex: 0 0 auto;
}
</style>