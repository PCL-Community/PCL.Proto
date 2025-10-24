<script setup lang="ts">
import type { INavItem } from '@/types/naviOptions'
import IconRefresh from '@/components/icons/side/IconRefresh.vue'
import { RouterLink } from 'vue-router'
defineProps<INavItem>()
</script>

<template>
  <component
    :is="linkto ? RouterLink : 'a'"
    :to="linkto"
    @click="clickCallback"
    :class="{ custom: !linkto, 'sidenav-line': true }"
  >
    <svg class="indicator" width="4" height="23" viewBox="0 0 4 23" v-if="linkto">
      <line
        x1="2"
        y1="2"
        x2="2"
        y2="21"
        stroke="currentColor"
        stroke-width="4"
        stroke-linecap="round"
      />
    </svg>

    <i class="side-nav-icon" v-if="icon"><icon /></i>
    <p>{{ text }}</p>
    <i class="refresh-icon button-animated" v-if="linkto"><IconRefresh /></i>
  </component>
</template>

<style scoped>
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
  padding-inline-end: 10px;
}

a:hover {
  background-color: var(--color-tint-lighter);
}

a.router-link-active {
  color: var(--color-tint);
}

a.custom {
  /* left: 11px gap + 4px width */
  padding-inline: 15px 22px;
  cursor: pointer;
}

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
  color: var(--color-tint-light);
  opacity: 0;
  transition: 0.4s;
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
</style>
