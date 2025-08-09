<script lang="ts" setup>
import { computed } from 'vue'

type AvatarInputType = 'url' | 'username' | 'uuid'
const props = withDefaults(defineProps<{ type?: AvatarInputType; src: string }>(), {
  type: 'username',
  src: 'AMagicPear',
})

const skinUrl = computed(() => {
  switch (props.type) {
    case 'username':
      return `https://minotar.net/skin/${props.src}`
    case 'uuid':
      return `https://crafatar.com/skins/${props.src}`
    case 'url':
    default:
      return props.src
  }
})
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
