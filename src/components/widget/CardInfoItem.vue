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
    pixelatedIcon?: boolean
  }>(),
  {
    isGameVersion: false,
    hoverEffect: true,
    roundImg: false,
    pixelatedIcon: false,
  },
)
</script>

<template lang="pug">
    .gameinfo-container(@click="click?.()" :class="{'game-info': isGameInfo, 'hover-effect': hoverEffect, 'round-img': roundImg, 'clickable': click }")
        .left
            img(:src="icon" v-if="icon" :style="{ 'image-rendering': pixelatedIcon ? 'pixelated' : 'auto' }")
            .text
                .title {{ title }}
                .subtitle {{ subtitle }}
        .right
            a: i(title="另存为" v-if="isGameInfo"): IconButtonSave
            a(:href="'https://zh.minecraft.wiki/w/Special:Search?search=' + title" target="_blank"): i(title="更新日志" v-if="isGameInfo"): IconInfo
            a: i(title="下载服务端" v-if="isGameInfo"): IconServer
            a(v-if="btn" :href="btn.link"): PButton(:inline="true") {{btn.text}}

</template>

<style lang="scss" scoped>
.gameinfo-container {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  padding: 6px;
  border-radius: 4px;
  gap: 6px;
  transition: background-color 0.4s ease;

  &.clickable {
    transition:
      scale 0.2s,
      background-color 0.4s ease;
  }

  &.hover-effect:hover {
    background-color: var(--color-tint-lighter);
  }

  &.clickable:active {
    scale: 0.98;
    background-color: var(--half-transparent-blue);
  }

  > .right {
    > a {
      > i {
        transition: color 0.4s;
        color: var(--color-tint-light);

        &:hover {
          color: var(--color-tint);
        }
      }

      svg {
        width: 15px;
      }
    }
  }

  &.game-info > .right {
    margin-right: 6px;
    opacity: 0;
    transition: opacity 0.4s ease;
    display: flex;
    align-items: center;
    gap: 9.5px;
  }

  > .left {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 6px;

    > img {
      width: 30px;
      height: 30px;
      /* fix #3 图标模糊 */
      /* image-rendering: pixelated; */
    }
  }

  &.round-img > .left > img {
    border-radius: 50%;
  }

  &.hover-effect:hover > .right {
    opacity: 1;
  }
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
