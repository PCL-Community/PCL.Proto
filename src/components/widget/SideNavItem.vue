<script setup lang="ts">
import type { INavItem } from '@/options/naviOptions';
import IconRefresh from '@/components/icons/side/IconRefresh.vue';
import { useRoute } from 'vue-router';
defineProps<INavItem>()
</script>

<template>
    <RouterLink :to="linkto ?? '/home'">
        <svg width="4" height="23" viewBox="0 0 4 23" class="indicator">
            <line x1="2" :y1="2" x2="2" :y2="21" stroke="currentColor" stroke-width="4" stroke-linecap="round"></line>
        </svg>
        <i class="side-nav-icon">
            <component :is="icon" />
        </i>
        <p>{{ itemName }}</p>
        <i class="refresh-icon button-animated">
            <component :is="IconRefresh" />
        </i>
    </RouterLink>
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

a:hover i.refresh-icon {
    opacity: 1;
}

.side-nav-icon {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
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