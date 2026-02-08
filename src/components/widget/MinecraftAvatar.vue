<script lang="ts" setup>
defineProps<{ skinUrl?: string }>()
</script>

<template>
  <div id="avatar">
    <img :src="skinUrl" crossorigin="anonymous" class="avatar-img" />
    <img :src="skinUrl" crossorigin="anonymous" class="avatar-hat" />
  </div>
</template>

<style scoped>
#avatar {
  width: 58px;
  height: 58px;
  margin: 6px;
  overflow: hidden;
  display: grid;
  place-items: center;
}

.avatar-img {
  grid-area: 1 / 1;
  width: 8px;
  height: 8px;
  object-fit: none;
  object-position: -8px -8px;
  transform: scale(5.4);
  image-rendering: pixelated;
  box-shadow: 0 0 1px rgba(0, 0, 0, 0.2);
}

@media (prefers-color-scheme: dark) {
  .avatar-img {
    box-shadow: 0 0 1px rgba(255, 255, 255, 0.5);
  }
}

.avatar-hat {
  grid-area: 1 / 1;
  width: 7.99px;
  height: 7.99px;
  object-fit: none;
  object-position: -40px -8px;
  transform: scale(6.1);
  image-rendering: pixelated;
}
</style>
