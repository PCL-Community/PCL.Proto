<script setup lang="ts">
import { pclText } from '@/api/pclLocale'

defineProps<{ error?: string | null }>()
defineEmits<{ retry: [] }>()
</script>

<template>
  <p v-if="error" role="alert">
    {{ error }}
    <button type="button" @click="$emit('retry')">
      {{ pclText('Launch.Homepage.LoadFailed.Retry') }}
    </button>
  </p>
</template>
