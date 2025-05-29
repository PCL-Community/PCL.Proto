<template>
    <div class="app-layout">
        <!-- 左侧导航栏区域 -->
        <div class="side-nav" :style="{ width: sidebarWidth + 'px' }" ref="sidebarWrapper">
            <transition name="sidebar">
                <div class="sidebar-content" ref="sidebarContent">
                    <RouterView name="sidebar" />
                </div>
            </transition>
        </div>

        <!-- 右侧内容区域 -->
        <div class="main-content">
            <RouterView />
        </div>
    </div>
</template>

<script setup>
import { onMounted, onUpdated, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

const sidebarContent = ref(null)
const sidebarWrapper = ref(null)
const sidebarWidth = ref(0)
const route = useRoute()

const updateSidebarWidth = () => {
    if (sidebarContent.value) {
        sidebarWidth.value = sidebarContent.value.offsetWidth
    }
}

onMounted(() => {
    updateSidebarWidth()
})

onUpdated(() => {
    updateSidebarWidth()
})

watch(() => route.fullPath, () => {
    // 延迟执行以等待内容渲染
    setTimeout(updateSidebarWidth, 100)
})
</script>

<style scoped>
.app-layout {
    display: flex;
    height: 100%;
}

.side-nav {
    background-color: #f5f5f5;
    transition: width 0.3s ease;
    overflow: hidden;
}

.sidebar-content {
    padding: 1rem;
}

.main-content {
    flex: 1;
    padding: 1rem;
}
</style>
