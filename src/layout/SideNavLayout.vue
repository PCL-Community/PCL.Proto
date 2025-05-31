<script setup lang="ts">
import { sideNavState } from '@/router/windowState';
import { onMounted, onUnmounted, ref } from 'vue';
import SideGroup from '@/components/widget/SideGroup.vue'
import { type INavItemGroup } from '@/router/naviOptions';
defineProps<{
    sideNavGroups: INavItemGroup[]
}>()

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

    let sidenavLines = document.querySelectorAll('.sidenav-line');
    sidenavLines.forEach((line) => { console.log(line.textContent + ': ' + line.classList.value) })
})

onUnmounted(() => observer?.disconnect())
</script>

<template>
    <div class="view-content">
        <aside ref="asideRef">
            <SideGroup v-for="group in sideNavGroups" :title="group.title" :content="group.content" />
        </aside>
        <article class="subview">
            <slot></slot>
        </article>
    </div>
</template>

<style scoped>
.view-content {
    display: flex;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
}

article {
    flex: 1 1 0;
    overflow-y: auto;
    padding-bottom: 76px;
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