<script lang="ts" setup>
defineProps<{ label: string; id: string }>()
const model = defineModel<boolean>({ default: false })
</script>

<template>
  <div class="option">
    <input type="checkbox" :id="id" class="checkable" v-model="model" />
    <label :for="id">{{ label }}</label>
  </div>
</template>

<style lang="css" scoped>
.option {
  margin-block: 4px;
}
</style>
