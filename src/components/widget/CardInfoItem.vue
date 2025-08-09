<script setup lang="ts">
import IconButtonSave from '../icons/control/IconButtonSave.vue'
import IconInfo from '../icons/control/IconInfo.vue'
import IconServer from '../icons/control/IconServer.vue'
import PButton from './PButton.vue'

withDefaults(
  defineProps<{
    icon?: string
    title: string
    subtitle?: string
    isGameInfo?: boolean
    roundImg?: boolean
    hoverEffect?: boolean
    btn?: {
      text: string
      link: string
    }
    click?: () => void
  }>(),
  {
    isGameVersion: false,
    hoverEffect: true,
    roundImg: false,
  },
)
</script>

<template lang="pug">
    .gameinfo-container(@click="click?.()" :class="{'game-info': isGameInfo, 'hover-effect': hoverEffect, 'round-img': roundImg, 'clickable': click }")
        .left
            img(:src="icon" v-if="icon")
            .text
                .title {{ title }}
                .subtitle {{ subtitle }}
        .right
            IconButtonSave(v-if="isGameInfo")
            IconInfo(v-if="isGameInfo")
            IconServer(v-if="isGameInfo")
            a(v-if="btn" :href="btn.link"): PButton(:inline="true") {{btn.text}}
            
</template>

<style lang="css" scoped>
.left {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 6px;
}

.left > img {
  width: 30px;
  height: 30px;
  image-rendering: pixelated;
}

.round-img > .left > img {
  border-radius: 50%;
}

.gameinfo-container.game-info > .right {
  margin-right: 6px;
  color: var(--color-tint-light);
  opacity: 0;
  transition: opacity 0.4s ease;
  display: flex;
  align-items: center;
  gap: 9.5px;
}

.right > svg {
  width: 14px;
}

.hover-effect:hover > .right {
  opacity: 1;
}

.gameinfo-container {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  padding: 6px;
  border-radius: 4px;
  gap: 6px;
  transition: background-color 0.4s ease;
}

.gameinfo-container.clickable {
  cursor: pointer;
}

.gameinfo-container.hover-effect:hover {
  background-color: var(--color-tint-lighter);
}

.title {
  font-size: 13px;
  line-height: 1.1rem;
}

.subtitle {
  font-size: 11px;
  color: var(--color-text-grey);
  line-height: normal;
}
</style>
