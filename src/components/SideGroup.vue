<script lang="ts" setup>
import IconOverview from './icons/side/IconOverview.vue';
import IconRefresh from './icons/side/IconRefresh.vue';
import IconDownload from './icons/header/IconDownload.vue';
</script>

<template>
    <div class="side-group">
        <RouterLink to="/download/game">
            <svg width="4" height="23" viewBox="0 0 4 23" class="indicator">
                <path stroke="currentColor" stroke-width="4" stroke-linecap="round" d="M2 2L2 21"></path>
            </svg>
            <i class="side-nav-icon">
                <component :is="IconOverview" />
            </i>
            <p>游戏下载1111</p>
            <i class="refresh-icon button-animated">
                <component :is="IconRefresh" />
            </i>
        </RouterLink>

        <RouterLink to="/download/manual">
            <svg width="4" height="23" viewBox="0 0 4 23" class="indicator">
                <path stroke="currentColor" stroke-width="4" stroke-linecap="round" d="M2 2L2 21"></path>
            </svg>
            <i class="side-nav-icon">
                <component :is="IconDownload" />
            </i>
            <p>手动下载</p>
            <i class="refresh-icon button-animated">
                <component :is="IconRefresh" />
            </i>
        </RouterLink>
    </div>
</template>

<style lang="css" scoped>
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

.side-group a:hover i.refresh-icon {
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
}
</style>