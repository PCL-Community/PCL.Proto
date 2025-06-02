<script lang="ts" setup>
import { computed } from 'vue';

type AvatarInputType = 'url' | 'username' | 'uuid'
const props = defineProps<{ type?: AvatarInputType, src: string }>()

const skinUrl = computed(() => {
    switch (props.type) {
        case 'username':
            return `https://minotar.net/skin/${props.src}`;
        case 'uuid':
            return `https://crafatar.com/skins/${props.src}`
        case 'url':
        default:
            return props.src;
    }
})
</script>

<template>
    <img :src="skinUrl" crossorigin="anonymous" class="avatar-img" />
</template>

<style scoped>
.avatar-img {
    width: 8px;
    height: 8px;
    object-fit: none;
    object-position: -8px -8px;
    transform: scale(5.625);
    image-rendering: pixelated;
}
</style>