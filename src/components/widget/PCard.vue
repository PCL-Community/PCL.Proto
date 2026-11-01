<script lang="ts" setup>
import { onUnmounted, ref, useTemplateRef, watch } from 'vue'
import IconUnfold from '../icons/control/IconUnfold.vue'

export type FoldStatus = 'unfold' | 'fold' | 'unfoldable'
export type FoldArrow = 'down' | 'right'

const props = withDefaults(
  defineProps<{
    hideTitle?: boolean
    defaultFoldStatus?: FoldStatus
    foldArrow?: FoldArrow
    overflowVisible?: boolean
    title?: string
    disabled?: boolean
  }>(),
  {
    hideTitle: false,
    defaultFoldStatus: 'unfoldable',
    foldArrow: 'down',
    overflowVisible: false,
  },
)

const foldState = ref<FoldStatus>(props.defaultFoldStatus)
// Keep opened content alive so folding preserves form and selection state.
const contentMounted = ref(props.defaultFoldStatus !== 'fold')
const cardHeight = ref<number>(40)
const mycardInnerRef = useTemplateRef<HTMLElement>('mycardInner')
const shouldSpring = ref<boolean>(true)

let observer: ResizeObserver | null = null

function toggleFold() {
  if (props.disabled) return
  switch (foldState.value) {
    case 'unfold':
      foldState.value = 'fold'
      // 卡片折叠状态固定为高度40
      // 在此处赋值是因为卡片内容有过渡时间
      // 切换时就赋值可以让卡片高度和内容的过渡同时进行
      shouldSpring.value = cardHeight.value < 500
      cardHeight.value = 40
      break
    case 'fold':
      contentMounted.value = true
      foldState.value = 'unfold'
      break
  }
}

watch(
  () => props.disabled,
  (disabled) => {
    if (disabled) collapse()
  },
)
function collapse() {
  if (foldState.value !== 'unfold') return
  shouldSpring.value = cardHeight.value < 500
  foldState.value = 'fold'
  cardHeight.value = 40
}
defineExpose({ collapse })

watch(
  [mycardInnerRef, contentMounted],
  ([inner, mounted]) => {
    observer?.disconnect()
    // Only opened, foldable cards need height measurement.
    if (!inner || !mounted || foldState.value === 'unfoldable') return
    observer = new ResizeObserver(() => {
      // 若非折叠状态需要根据内容调整高度
      // 为了方便使高度动画具有回弹动效，使用observer侦测内容来赋值而非让其自动撑开
      if (foldState.value != 'fold') {
        const height = inner.offsetHeight
        shouldSpring.value = height < 500
        cardHeight.value = height
      }
    })
    observer.observe(inner)
  },
  { flush: 'post' },
)

onUnmounted(() => observer?.disconnect())
</script>

<template>
  <div
    :class="[
      'mycard-container',
      foldState,
      {
        'overflow-visible': overflowVisible,
        'fold-disabled': disabled,
        'fold-arrow-right': foldArrow === 'right',
      },
    ]"
    :style="
      foldState === 'unfoldable'
        ? undefined
        : {
            height: `${cardHeight}px`,
            '--card-height-easing': shouldSpring ? 'cubic-bezier(0.34, 1.56, 0.64, 1)' : 'ease',
          }
    "
  >
    <div class="mycard" ref="mycardInner" :class="{ 'hide-title': hideTitle }">
      <header
        class="mycard-title"
        v-if="!hideTitle"
        :role="foldState !== 'unfoldable' && !disabled ? 'button' : undefined"
        :tabindex="foldState !== 'unfoldable' && !disabled ? 0 : undefined"
        :aria-expanded="foldState !== 'unfoldable' && !disabled ? foldState !== 'fold' : undefined"
        @keydown.enter.self="toggleFold"
        @keydown.space.self.prevent="toggleFold"
        @click="toggleFold"
      >
        <!-- 此处为兼容性设计：title插槽被设置时显示插槽内容，否则默认显示props中的title属性 -->
        <p>
          <slot name="title">{{ title }}</slot>
        </p>
        <div class="description" v-if="$slots.description">
          <slot name="description"></slot>
        </div>
        <div class="mycard-title-actions">
          <slot name="header-actions" />
          <i v-if="!disabled && foldState !== 'unfoldable'"><IconUnfold /></i>
        </div>
      </header>
      <Transition name="card-content">
        <section
          class="mycard-content"
          v-if="contentMounted && ($slots.content || $slots.default)"
          v-show="foldState == 'unfold' || foldState == 'unfoldable'"
        >
          <!-- 此处为兼容性设计：content被设置时显示content内容，否则显示默认插槽内容 -->
          <slot name="content"><slot></slot></slot>
        </section>
      </Transition>
    </div>
  </div>
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
  transition:
    height 0.4s var(--card-height-easing, ease),
    box-shadow 90ms linear;
  border-radius: 6px;
  background: var(--color-background-soft);
  box-shadow: 0px 0px 6px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.mycard-container.overflow-visible,
.mycard-container.overflow-visible > .mycard > .mycard-content {
  overflow: visible;
}

.mycard-container.fold {
  content-visibility: auto;
  contain-intrinsic-size: auto 40px;
}

.mycard-container:hover {
  box-shadow: 0px 0px 6px var(--color-tint-shadow);
}

.mycard-container:not(.unfoldable):not(.fold-disabled) .mycard-title {
  cursor: pointer;
}

.mycard {
  padding: 0;
}

.mycard.hide-title {
  /* 不知道为什么必须有上下的padding */
  padding: 0;
}

.mycard > .mycard-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--color-text);
  min-height: 40px;
  padding: 0 15px;
}

.mycard-title-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 0 0 auto;
}

.mycard-title-actions > i {
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 0;
  color: var(--color-text);
  /* CE MyCard：箭头旋转 250 ms，悬停配色 90 ms。 */
  transition:
    transform 250ms cubic-bezier(0.22, 1, 0.36, 1),
    color 90ms linear;
  flex: 0;
}

.mycard-container.unfold .mycard-title-actions > i {
  transform: rotate(180deg);
}

.mycard-container.fold.fold-arrow-right .mycard-title-actions > i {
  transform: rotate(-90deg);
}

.mycard-container.unfold.fold-arrow-right .mycard-title-actions > i {
  transform: rotate(-180deg);
}

.mycard > .mycard-title > p *,
.mycard > .mycard-title > p {
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0px;
  line-height: 14.06px;
  text-align: left;
  vertical-align: top;
  transition: color 90ms linear;
  flex: 1;
}

.mycard-container:hover > .mycard > .mycard-title {
  color: var(--ce-color-2);
}

.mycard > .mycard-content {
  font-size: 12px;
  margin: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.description {
  flex: 3;
}

.pcl-page .mycard-title {
  font-size: 13px;
}

.pcl-page .mycard-content {
  font-size: 13px;
}

@media (prefers-reduced-motion: reduce) {
  .mycard-container {
    transition: box-shadow 90ms linear;
  }
}
</style>
