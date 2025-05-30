<script setup lang="ts">

</script>

<template>

  <div>Download page
    <button v-for="i in 3" :key="i" @click="$emit('navButton', i - 1)">
      按钮{{ i - 1 }}
    </button>
  </div>
</template>
