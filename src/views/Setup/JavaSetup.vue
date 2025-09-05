<script lang="ts" setup>
import PCard from '@/components/widget/PCard.vue'
import { type IJavaRuntimeInfo, getJavaList, refreshJavaList } from '@/api/javaReq'
import { onMounted, ref } from 'vue'
import PButton from '@/components/widget/PButton.vue'
import sideTip from '@/composables/sideTip'
// import PlainTextInfoItem from '@/components/widget/PlainTextInfoItem.vue'
import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import { invoke } from '@tauri-apps/api/core'

const loading = ref(false)
const error = ref(null)
const javaList = ref<IJavaRuntimeInfo[]>()

// const { refreshJavaList, getJavaList } = JavaReq()

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
    javaList.value = await refreshJavaList()
    sideTip.show('Java 列表已成功刷新', 'default')
  } catch (err: any) {
    error.value = err.toString()
  } finally {
    loading.value = false
  }
}

async function manualAdd() {
  // sideTip.show('手动添加 Java 功能仅在本地客户端中可用', 'warn')
  let addedJavaPath = (await invoke('add_java')) as IJavaRuntimeInfo | null
  console.log('[java] added: ', addedJavaPath)
  if (addedJavaPath) {
    javaList.value?.push(addedJavaPath)
  }
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
        :key="runtime.directory_path"
        :title="`${runtime.is_jdk ? 'JDK' : 'JRE'} ${runtime.slug_version}(${runtime.version}) ${runtime.architecture} ${runtime.implementor ?? ''}`"
        :subtitle="runtime.directory_path"
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
