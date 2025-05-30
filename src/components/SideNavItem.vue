<script setup lang="ts">
defineProps<{ itemName?: string, isSelected?: Boolean }>()
import IconOverview from './icons/side/IconOverview.vue';
import IconRefresh from './icons/side/IconRefresh.vue';
</script>

<template>
    <div class="side-nav-item" :class="{ 'selected-blue': isSelected }">
        <svg :style="{ visibility: isSelected ? 'visible' : 'hidden' }" width="4" height="23" viewBox="0 0 4 23">
            <path stroke="currentColor" stroke-width="4" stroke-linecap="round" d="M2 2L2 21"></path>
        </svg>
        <i class="side-nav-icon">
            <slot>
                <IconOverview />
            </slot>
        </i>
        <p class="side-nav-text">{{ itemName }}</p>
        <i class="refresh-icon button-animated">
            <IconRefresh />
        </i>
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

.side-nav-item:hover i.refresh-icon {
    opacity: 1;
    /* visibility: visible; */
}

i.refresh-icon:hover {
    color: var(--color-titlebar);
}

.side-nav-item {
    width: 100%;
    display: flex;
    gap: 11px;
    align-items: center;
    justify-content: flex-start;
    font-size: 13px;
    height: 34px;
    background-color: transparent;
    transition: 0.4s;
    /* border-width: 1px 0 1px 0;
    border-color: transparent;
    border-style: solid; */
}

.side-nav-item:hover {
    background-color: var(--color-tint-lighter);
    /* border-color: var(--color-tint-lightist) */
}

.selected-blue {
    color: var(--color-selected);
}

.side-nav-icon {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
}
</style>