<script lang="ts" setup>
import { pclText } from '@/api/pclLocale'
import { ref, watch } from 'vue'
const props = defineProps<{ skinUrl?: string }>()
const fallback = `${import.meta.env.BASE_URL}pcl/images/Skins/Steve.png`
const source = ref(props.skinUrl || fallback)
watch(
  () => props.skinUrl,
  (value) => {
    source.value = value || fallback
  },
)
</script>

<template>
  <div class="skin">
    <img
      :src="source"
      @error="source = fallback"
      crossorigin="anonymous"
      class="avatar-img"
      :alt="pclText('Proto.Label.PlayerSkin')"
    />
    <img
      :src="source"
      @error="source = fallback"
      crossorigin="anonymous"
      class="avatar-hat"
      alt=""
    />
  </div>
</template>

<style scoped>
.skin {
  width: 64px;
  height: 64px;
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
  transform: scale(6);
  image-rendering: pixelated;
  box-shadow: 0 0 1px rgba(0, 0, 0, 0.2);
}

:global(.dark) .avatar-img {
  box-shadow: 0 0 1px rgba(255, 255, 255, 0.5);
}

.avatar-hat {
  grid-area: 1 / 1;
  width: 7.99px;
  height: 7.99px;
  object-fit: none;
  object-position: -40px -8px;
  transform: scale(7);
  image-rendering: pixelated;
}
</style>
