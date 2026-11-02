<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import PButton from './PButton.vue'

defineProps<{ error?: string | null }>()
defineEmits<{ retry: [] }>()
</script>

<template>
  <p v-if="error" role="alert">
    {{ error }}
    <PButton inline @click="$emit('retry')">
      {{ pclText('Launch.Homepage.LoadFailed.Retry') }}
    </PButton>
  </p>
</template>
