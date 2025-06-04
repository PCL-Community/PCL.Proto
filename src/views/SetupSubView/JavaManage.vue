<script lang="ts" setup>
import MyCard from '@/components/widget/MyCard.vue';
import { type IJavaRuntimeInfo, archMap } from '@/api/javaReq';
import JavaReq from '@/api/javaReq';
import { onMounted, ref } from 'vue';
import MyButton from '@/components/widget/MyButton.vue';
import sideTip from '@/composables/sideTip';

const loading = ref(false)
const error = ref(null)
const javaList = ref<IJavaRuntimeInfo[]>()

const { refreshJavaList, getJavaList } = JavaReq()

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

const refresh = async () => {
    error.value = null
    loading.value = true
    try {
        let status = await refreshJavaList();
        if (status.success != true)
            throw new Error('刷新 Java 列表失败');
        javaList.value = await getJavaList();
        sideTip.show('Java 列表已成功刷新', 'default');
    } catch (err: any) {
        error.value = err.toString();
    } finally {
        loading.value = false
    }
}

/** 
 * 未来如果用Tauri打包，可以使用 Tauri 的文件选择器
*/
function manualAdd() {
    sideTip.show('手动添加 Java 功能仅在本地客户端中可用', 'warn');
}

</script>

<template>
    <MyCard>
        <template #title>Java 列表 <span v-if="loading" id="loading">加载中...</span></template>
        <template #content>
            <p v-if="error">未能获取 Java 信息，请检查本地服务是否已经运行。</p>
            <p v-if="javaList?.length == 0">当前 Java 列表为空。</p>
            <div v-for="runtime in javaList" class="java-info">
                <p>{{ runtime.isJre ? "JRE" : "JDK" }} {{ runtime.slugVersion }}({{ runtime.version
                }})
                    {{ archMap[runtime.architecture] }} {{ runtime.implementor }}</p>
                <p class="java-info-c">{{ runtime.directoryPath }}</p>
            </div>
            <div class="refrsh-button-wrapper">
                <MyButton @click="manualAdd">手动添加</MyButton>
                <MyButton @click="refresh">刷新</MyButton>
            </div>
        </template>
    </MyCard>
</template>

<style scoped>
#loading {
    color: var(--color-text-grey);
    font-size: 0.9em;
}

.refrsh-button-wrapper {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 10px;
}

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