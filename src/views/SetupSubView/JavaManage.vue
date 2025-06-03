<script lang="ts" setup>
import MyCard from '@/components/widget/MyCard.vue';
import { getJavaList, type IJavaRuntimeInfo, archMap } from '@/api/javaReq';
import { onActivated, onMounted, ref, watch } from 'vue';
import { useRoute } from 'vue-router';

const loading = ref(false)
const error = ref(null)
const javaList = ref<IJavaRuntimeInfo[]>()

onMounted(async () => {
    error.value = null
    loading.value = true
    try {
        javaList.value = await getJavaList();
    } catch (err: any) {
        error.value = err.toString();
    } finally {
        loading.value = false
    }
})

</script>

<template>
    <MyCard>
        <template #title>Java 列表</template>
        <template #content>
            <p v-if="loading">Loading...</p>
            <p v-if="error">未能获取 Java 信息，请检查本地服务是否已经运行。</p>
            <div v-for="runtime in javaList" class="java-info">
                <p class="java-info-h">{{ runtime.isJre ? "JRE" : "JDK" }} {{ runtime.slugVersion }}({{ runtime.version
                    }})
                    {{ archMap[runtime.architecture] }} {{ runtime.implementor }}</p>
                <p>{{ runtime.directoryPath }}</p>
            </div>
        </template>
    </MyCard>
</template>

<style scoped>
.java-info {
    padding: 4px;
    border-radius: 4px;
    transition: background-color 0.4s ease;
}

.java-info:hover {
    background-color: var(--color-tint-lighter);
}

.java-info-h {
    font-size: 15px;
}
</style>