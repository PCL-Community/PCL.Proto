<script setup lang="ts">
import PButton from './PButton.vue'
import PSelectionIndicator from './PSelectionIndicator.vue'
import PListActions from './PListActions.vue'
import { openExternal } from '@/util/externalLink'

withDefaults(
  defineProps<{
    icon?: string
    title: string
    subtitle?: string
    tags?: string[]
    selected?: boolean
    roundImg?: boolean
    hoverEffect?: boolean
    btn?: {
      text: string
      link: string
    }
    clickable?: boolean
    pixelatedIcon?: boolean
  }>(),
  {
    hoverEffect: true,
    clickable: false,
    roundImg: false,
    pixelatedIcon: false,
    selected: undefined,
  },
)
const emit = defineEmits<{ click: [event: MouseEvent | KeyboardEvent] }>()
</script>

<template lang="pug">
    .gameinfo-container(@click="clickable && emit('click', $event)" :role="clickable ? 'button' : undefined" :tabindex="clickable ? 0 : undefined" :aria-pressed="selected" @keydown.enter.self="clickable && emit('click', $event)" @keydown.space.self.prevent="clickable && emit('click', $event)" :class="{'hover-effect': hoverEffect, 'round-img': roundImg, 'clickable': clickable, 'selectable': selected !== undefined, selected }")
        PSelectionIndicator.list-selection(v-if="selected !== undefined")
        .left
            slot(name="icon")
                img(:src="icon" v-if="icon" :style="{ 'image-rendering': pixelatedIcon ? 'pixelated' : 'auto' }")
            .text
                .title
                    slot(name="title")
                        span {{ title }}
                    span.list-tag(v-for="tag in tags" :key="tag") {{ tag }}
                .subtitle {{ subtitle }}
        .right(@click.stop)
            slot(name="actions")
            PListActions(v-if="$slots['icon-actions']")
                slot(name="icon-actions")
            PButton(v-if="btn" compact @click="openExternal(btn.link)") {{btn.text}}

</template>

<style lang="scss" scoped>
.gameinfo-container {
  display: flex;
  min-height: var(--list-min-height, auto);
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  padding: 6px;
  border-radius: 4px;
  background-color: var(--color-tint-rest);
  gap: 6px;
  /* CE MyListItem：移出 180 ms、指向 120 ms；按下缩放 108 ms，松开 144/540 ms。 */
  transition: background-color 180ms linear;

  &.selectable {
    position: relative;
    padding-left: 14px;
  }

  > .list-selection {
    position: absolute;
    left: 0;
    top: calc(50% - 11.5px);
  }

  &.clickable {
    transition:
      scale 540ms cubic-bezier(0.33, 1, 0.68, 1),
      background-color 180ms linear;
  }

  &.hover-effect:hover {
    background-color: var(--color-tint-lighter);
    transition-duration: 120ms;
  }

  &.clickable:hover {
    transition-duration: 144ms, 120ms;
  }

  &.clickable:active:not(:has(.right:active)) {
    scale: 0.98;
    background-color: var(--half-transparent-blue);
    transition-duration: 108ms, 120ms;
  }

  > .right {
    display: flex;
    align-items: center;
    gap: var(--list-actions-gap, 6px);
    > a {
      > i {
        transition: color 150ms linear;
        color: var(--color-tint-light);

        &:hover {
          color: var(--color-tint);
          transition-duration: 120ms;
        }
      }

      svg {
        width: 15px;
      }
    }
  }

  > .left {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 6px;
    min-width: 0;

    > .text {
      min-width: 0;
      overflow-wrap: anywhere;
    }

    > img {
      width: var(--list-icon-size, 30px);
      height: var(--list-icon-size, 30px);
      border-radius: var(--list-icon-radius, 0);
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
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 3px 6px;
  font-size: var(--list-title-size, 13px);
  line-height: 1.1rem;
}

.list-tag {
  padding: 1px 3px;
  border-radius: 3px;
  background: var(--ce-color-6);
  color: var(--ce-color-2);
  font-size: 11px;
  line-height: 14px;
}

.subtitle {
  font-size: 11px;
  color: var(--color-text-grey);
  line-height: normal;
}
</style>
