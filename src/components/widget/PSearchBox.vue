<script setup lang="ts">
import { ref } from 'vue'
import PButton from './PButton.vue'
import PCard from './PCard.vue'
import IconSearch from '@/assets/icons/IconSearch.svg'
defineProps<{ placeholder?: string }>()
const emit = defineEmits(['search'])
const query = ref<string>()
</script>

<template>
  <PCard hide-title>
    <section class="search-card">
      <IconSearch class="search-icon" />
      <input
        type="text"
        :placeholder="placeholder"
        class="search-input"
        v-model="query"
        v-on:keydown.enter="emit('search', query)"
      />
      <PButton inline type="tint" @click="emit('search', query)">搜索</PButton>
    </section>
  </PCard>
</template>

<style scoped>
.search-card {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  gap: 10px;
}

.search-icon {
  width: 18px;
}

.search-input {
  flex: 1;
  font-size: 15px;
  text-align: left;
  color: var(--color-text-black);
  font-family: 'PCL-English';
  border: none;
  outline: none;
  background-color: transparent;
}
</style>
