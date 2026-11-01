<script setup lang="ts">
import type { INavItem } from '@/types/naviOptions'
import { RouterLink } from 'vue-router'
import { inject } from 'vue'
import PSelectionIndicator from './PSelectionIndicator.vue'
const runAction = inject<(kind: string) => void>('navAction')
defineProps<INavItem>()
</script>

<template>
  <div class="sidenav-line">
    <component
      :is="linkto ? RouterLink : 'a'"
      :to="linkto"
      @click="clickCallback"
      :class="{ custom: !linkto }"
    >
      <PSelectionIndicator v-if="linkto" />

      <i class="side-nav-icon" v-if="icon"><icon /></i>
      <div class="side-nav-text">
        <p>{{ text }}</p>
        <p v-if="subtitle" class="side-nav-subtitle" :title="subtitle">{{ subtitle }}</p>
      </div>
    </component>
    <button
      v-if="action"
      class="refresh-icon pcl-icon-button"
      type="button"
      :title="action.title"
      :aria-label="action.title"
      @click="runAction?.(action.kind)"
    >
      <component :is="action.icon" />
    </button>
  </div>
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
  height: 36px;
  background-color: var(--color-tint-rest);
  transition: 0.4s;
  padding-inline-end: 40px;
}

.sidenav-line:hover > a {
  background-color: var(--color-tint-lighter);
}

a.router-link-active {
  color: var(--color-tint);
}

a.custom {
  /* left: 11px gap + 4px width */
  padding-inline-start: 15px;
  cursor: pointer;
}

.sidenav-line {
  position: relative;
}
.side-nav-text {
  min-width: 0;
}
.side-nav-subtitle {
  max-width: 155px;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 11px;
  color: var(--color-text-grey);
}
a:has(.side-nav-subtitle) {
  height: 40px;
}
.refresh-icon {
  position: absolute;
  right: 8px;
  top: 7px;
  width: 22px;
  height: 22px;
  padding: 3px;
  opacity: 0;
  transition:
    opacity 400ms linear,
    color 150ms linear,
    transform 250ms cubic-bezier(0.34, 2, 0.64, 1);
}

.sidenav-line:has(a.router-link-active):hover .refresh-icon,
.sidenav-line .refresh-icon:not(:disabled):focus-visible {
  opacity: 1;
  transition-duration: 400ms, 120ms, 250ms;
}

.sidenav-line .refresh-icon:not(:disabled):active {
  transform: scale(0.8);
  transition-duration: 400ms, 150ms, 400ms;
  transition-timing-function: linear, linear, cubic-bezier(0.25, 1, 0.5, 1);
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
