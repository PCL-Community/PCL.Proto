<script lang="ts" setup>
import MyCard from '@/components/widget/MyCard.vue';
import { getJavaList, type IJavaRuntimeInfo, archMap } from '@/api/javaReq';
import { onMounted, ref } from 'vue';

const loading = ref(false)
const error = ref(null)
const javaList = ref<IJavaRuntimeInfo[]>()

onMounted(async () => {
    error.value = null
    loading.value = true
    try {
        // await new Promise(resolve => setTimeout(resolve, 3000));
        // throw new Error('模拟获取 Java 信息失败'); // 模拟出错
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
            <p v-if="loading">加载中...</p>
            <p v-if="error">未能获取 Java 信息，请检查本地服务是否已经运行。</p>
            <div v-for="runtime in javaList" class="java-info">
                <p>{{ runtime.isJre ? "JRE" : "JDK" }} {{ runtime.slugVersion }}({{ runtime.version
                    }})
                    {{ archMap[runtime.architecture] }} {{ runtime.implementor }}</p>
                <p class="java-info-c">{{ runtime.directoryPath }}</p>
            </div>
        </template>
    </MyCard>
</template>

<style scoped>
.java-info {
    padding: 4px 6px;
    border-radius: 4px;
    transition: background-color 0.4s ease;
}

.java-info:hover {
    background-color: var(--color-tint-lighter);
}

.java-info-c {
    color: var(--color-text-grey);
}
</style>