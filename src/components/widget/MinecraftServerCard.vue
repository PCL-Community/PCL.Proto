<script lang="ts" setup>
import { onBeforeMount } from 'vue'

const props = defineProps<{ url: string }>()

onBeforeMount(() => {
  console.log('MinecraftServerCard is about to be mounted')
  console.log(props.url)
})
</script>

<template>
  <div class="server-card"></div>
</template>

<style scoped>
.server-card {
  background-color: aquamarine;
  height: 60px;
  border-radius: 4px;
}
</style>
