<script lang="ts" setup>
import { ref, useAttrs, useTemplateRef, watch } from 'vue'
import IconUnfold from '../icons/control/IconUnfold.vue'
import { observeCardResize } from '@/util/cardResize'

export type FoldStatus = 'unfold' | 'fold' | 'unfoldable'
export type FoldArrow = 'down' | 'right'

defineOptions({ inheritAttrs: false })
const attrs = useAttrs()
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
const mycardInnerRef = useTemplateRef<HTMLElement>('mycardInner')
const cardRef = useTemplateRef<HTMLElement>('card')

function toggleFold() {
  if (props.disabled) return
  switch (foldState.value) {
    case 'unfold':
      foldState.value = 'fold'
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
  foldState.value = 'fold'
}
defineExpose({ collapse })

watch(
  [mycardInnerRef, cardRef],
  ([inner, card], _, onCleanup) => {
    if (inner && card) onCleanup(observeCardResize(inner, card))
  },
  { flush: 'post' },
)
</script>

<template>
  <div
    ref="card"
    v-bind="attrs"
    :class="[
      'mycard-container',
      foldState,
      {
        'overflow-visible': overflowVisible,
        'fold-disabled': disabled,
        'fold-arrow-right': foldArrow === 'right',
      },
    ]"
    :style="foldState === 'fold' ? { height: 'auto' } : undefined"
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
      <section
        class="mycard-content"
        v-if="contentMounted && ($slots.content || $slots.default)"
        :inert="foldState === 'fold' || undefined"
        :aria-hidden="foldState === 'fold' || undefined"
      >
        <!-- 此处为兼容性设计：content被设置时显示content内容，否则显示默认插槽内容 -->
        <slot name="content"><slot></slot></slot>
      </section>
    </div>
  </div>
</template>

<style>
.mycard-container {
  flex-shrink: 0;
  transition: box-shadow 90ms linear;
  border-radius: 6px;
  background: var(--color-background-soft);
  box-shadow: 0px 0px 6px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.mycard-container.overflow-visible:not(.fold),
.mycard-container.overflow-visible:not(.fold) > .mycard > .mycard-content {
  overflow: visible;
}

.mycard-container:hover {
  box-shadow: 0px 0px 6px var(--color-tint-shadow);
}

.mycard-container:not(.unfoldable):not(.fold-disabled) .mycard-title {
  cursor: pointer;
}

.mycard {
  position: relative;
  padding: 0;
}

.mycard-container.fold .mycard-content {
  position: absolute;
  width: 100%;
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
