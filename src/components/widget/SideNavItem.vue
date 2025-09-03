<script setup lang="ts">
import type { INavItem } from '@/types/naviOptions'
import IconRefresh from '@/components/icons/side/IconRefresh.vue'
defineProps<INavItem>()
</script>

<template lang="pug">
    RouterLink(:to="linkto ?? '/home'")
        svg.indicator(width="4" height="23" viewBox="0 0 4 23")
            line(x1="2" y1="2" x2="2" y2="21" stroke="currentColor" stroke-width="4" stroke-linecap="round")

        i.side-nav-icon(v-if="icon")
            component(:is="icon")

        p {{ text }}
        i.refresh-icon.button-animated
            component(:is="IconRefresh")

</template>

<style scoped>
.indicator {
  visibility: hidden;
  transform: scaleY(0);
  transform-origin: center;
  transition: none;
}

a.router-link-active .indicator {
  visibility: visible;
  transform: scaleY(1);
  transition: transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}

i.refresh-icon {
  margin-left: auto;
  color: var(--light-blue);
  opacity: 0;
  transition: 0.4s;
  margin-right: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}

i.refresh-icon:hover {
  color: var(--color-titlebar);
}

a.router-link-active:hover i.refresh-icon {
  opacity: 1;
}

.side-nav-icon {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 修复Safari图标不显示的问题 */
:deep(.side-nav-icon > svg) {
  max-width: 100%;
  max-height: 100%;
}

a {
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

a:hover {
  background-color: var(--color-tint-lighter);
}

a.router-link-active {
  color: var(--color-tint);
}
</style>
