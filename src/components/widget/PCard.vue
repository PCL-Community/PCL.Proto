<script lang="ts" setup>
import { onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue'
import IconUnfold from '../icons/control/IconUnfold.vue'

export type FoldStatus = 'unfold' | 'fold' | 'unfoldable'

const props = withDefaults(
  defineProps<{
    hideTitle?: boolean
    defaultFoldStatus?: FoldStatus
    swapLogoRight?: boolean
    title?: string
  }>(),
  {
    hideTitle: false,
    defaultFoldStatus: 'unfoldable',
    swapLogoRight: false,
  },
)

const foldState = ref<FoldStatus>(props.defaultFoldStatus)

const cardHeight = ref<number>(40)

const mycardInnerRef = useTemplateRef<HTMLElement>('mycardInner')
const containerRef = useTemplateRef<HTMLDivElement>('container')

let observer: ResizeObserver | null = null

function SwitchFoldState() {
  switch (foldState.value) {
    case 'unfold':
      foldState.value = 'fold'
      // 卡片折叠状态固定为高度40
      // 在此处赋值是因为卡片内容有过渡时间
      // 切换时就赋值可以让卡片高度和内容的过渡同时进行
      cardHeight.value = 40
      break
    case 'fold':
      foldState.value = 'unfold'
      break
  }
}

onMounted(() => {
  observer = new ResizeObserver(() => {
    // 若非折叠状态需要根据内容调整高度
    // 为了方便使高度动画具有回弹动效，使用observer侦测内容来赋值而非让其自动撑开
    if (foldState.value != 'fold') cardHeight.value = mycardInnerRef.value!.offsetHeight
  })
  observer.observe(mycardInnerRef.value!)
})

watch(cardHeight, (newH, oldH) => {
  let out = newH - oldH > 0 ? 12 : -12
  containerRef.value?.animate(
    [{ height: oldH + 'px' }, { height: newH + out + 'px' }, { height: newH + 'px' }],
    {
      duration: Math.log(1 + Math.abs(newH - oldH)) * 80,
      easing: 'ease-in-out',
    },
  )
})

onUnmounted(() => observer?.disconnect())

// allow outside state control
defineExpose({ SwitchFoldState })
</script>

<template lang="pug">
.mycard-container(:class="foldState" ref="container")
    .mycard(ref="mycardInner" :class="{'hide-title': hideTitle }")
        header.mycard-title(v-if="!hideTitle" @click="SwitchFoldState")
            //- 此处为兼容性设计：title插槽被设置时显示插槽内容，否则默认显示props中的title属性
            p: slot(name="title") {{title}}
            .description(v-if="$slots.description"): slot(name="description")
            i: IconUnfold()
        Transition(name="card-content")
            section.mycard-content(v-if="$slots.content || $slots.default" v-show="foldState == 'unfold' || foldState == 'unfoldable'")
                //- 此处为兼容性设计：content被设置时显示content内容，否则显示默认插槽内容
                slot(name="content"): slot

</template>

<style>
.card-content-enter-active,
.card-content-leave-active {
  transition: opacity 0.2s;
}

.card-content-enter-from,
.card-content-leave-to {
  opacity: 0;
}

.mycard-container {
  flex-shrink: 0;
  /* height: v-bind("cardHeight + 'px'"); */
  transition: box-shadow 0.4s;
  /* height 0.4s cubic-bezier(0.4, 1.4, 0.6, 1), */
  /* height 0.3s ease-out, */
  border-radius: 6px;
  background: var(--color-background-soft);
  box-shadow: 0px 0px 6px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.mycard-container:hover {
  box-shadow: 0px 0px 6px var(--color-tint-shadow);
}

.mycard-container:not(.unfoldable) .mycard-title {
  cursor: pointer;
}

.mycard-container.unfoldable .mycard-title > i {
  visibility: hidden;
}

.mycard {
  padding: 8px 14px;
}

.mycard.hide-title {
  /* 不知道为什么必须有上下的padding */
  padding: 1px 6px;
}

.mycard > .mycard-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--color-text);
  height: fit-content;
}

.mycard > .mycard-title > i {
  color: var(--color-text);
  transition: transform 0.4s ease;
  flex: 0;
}

.mycard-container.unfold .mycard-title > i {
  transform: rotate(180deg);
}

.mycard > .mycard-title > p *,
.mycard > .mycard-title > p {
  font-size: 12px;
  font-weight: bold;
  letter-spacing: 0px;
  line-height: 14.06px;
  text-align: left;
  vertical-align: top;
  transition: color 0.4s;
  flex: 1;
}

.mycard-container:hover > .mycard > .mycard-title {
  color: var(--color-tint);
}

.mycard > .mycard-content {
  font-size: 12px;
  margin: 9px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.description {
  flex: 3;
}
</style>
