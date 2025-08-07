<script lang="ts" setup>
import PCard from '@/components/widget/PCard.vue'
import { type IJavaRuntimeInfo, archMap } from '@/api/javaReq'
import JavaReq from '@/api/javaReq'
import { onMounted, ref } from 'vue'
import PButton from '@/components/widget/PButton.vue'
import sideTip from '@/composables/sideTip'
// import PlainTextInfoItem from '@/components/widget/PlainTextInfoItem.vue'
import CardInfoItem from '@/components/widget/CardInfoItem.vue'

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
    javaList.value = await getJavaList()
  } catch (err: any) {
    error.value = err.toString()
  } finally {
    loading.value = false
  }
})

const refresh = async () => {
  error.value = null
  loading.value = true
  try {
    let status = await refreshJavaList()
    if (status.success != true) throw new Error('刷新 Java 列表失败')
    javaList.value = await getJavaList()
    sideTip.show('Java 列表已成功刷新', 'default')
  } catch (err: any) {
    error.value = err.toString()
  } finally {
    loading.value = false
  }
}

/**
 * 未来如果用Tauri打包，可以使用 Tauri 的文件选择器
 */
function manualAdd() {
  sideTip.show('手动添加 Java 功能仅在本地客户端中可用', 'warn')
}
</script>

<template>
  <PCard>
    <template #title>Java 列表 <span v-if="loading" id="loading">加载中...</span></template>
    <template #content>
      <p v-if="error">未能获取 Java 信息，请检查本地服务是否已经运行。</p>
      <p v-if="javaList?.length == 0">当前 Java 列表为空。</p>
      <CardInfoItem
        v-for="runtime in javaList"
        :title="`${runtime.isJre ? 'JRE' : 'JDK'} ${runtime.slugVersion}(${runtime.version}) ${archMap[runtime.architecture]} ${runtime.implementor}`"
        :subtitle="runtime.directoryPath"
      />
      <div class="refrsh-button-wrapper">
        <PButton :click="manualAdd">手动添加</PButton>
        <PButton :click="refresh">刷新</PButton>
      </div>
    </template>
  </PCard>
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
</style>
