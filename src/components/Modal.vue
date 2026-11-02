<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import { useModal } from '@/composables/useModal'
import PButton from './widget/PButton.vue'
import PListItem from './widget/PListItem.vue'
import PSvgIcon from './widget/PSvgIcon.vue'
import type { ModalButtonOption } from '@/types/modal'

const { isOpen, options, close, inputValue, selectedValue } = useModal()
const handleButtonClick = async (btn: ModalButtonOption) => {
  if (btn.operation) await btn.operation()
  else close(false)
}
</script>

<template lang="pug">
  Transition(name="modal")
    .modal-mask(v-if="isOpen")
      .modal-container
        .modal-header
          h2 {{ options.title ?? pclText('Proto.Label.UntitledDialog') }}

        .modal-divider

        .modal-body
          p(v-if="options.content" style="white-space: pre-wrap") {{ options.content }}

          input.modal-input(
            v-if="options.showInput"
            v-model="inputValue"
          )

          .modal-select(v-if="options.selectOptions")
            PListItem(
              v-for="option in options.selectOptions"
              :key="option.value"
              :title="option.label"
              clickable
              :selected="selectedValue === option.value"
              @click="selectedValue = option.value"
            )
              template(#icon)
                PSvgIcon(v-if="option.icon" :name="option.icon" :size="24")

        .modal-footer
          template(v-if="options.selectOptions")
            PButton(
              type="tint"
              :disabled="!selectedValue"
              @click="() => close(true)"
            ) {{ options.confirmText ?? pclText('Common.Action.Confirm') }}
            PButton(@click="() => close(false)" type="warn") {{ options.cancelText ?? pclText('Common.Action.Cancel') }}
          template(v-else-if="options.buttons && options.buttons.length")
            PButton(
              v-for="(btn, idx) in options.buttons"
              :key="idx"
              :type="btn.type"
              @click="() => handleButtonClick(btn)"
            ) {{ btn.content }}
          template(v-else)
            PButton(@click="() => close(true)" type="tint") {{ pclText('Proto.Action.Confirm') }}
            PButton(@click="() => close(false)" type="warn") {{ pclText('Common.Action.Cancel') }}

</template>

<style scoped>
.modal-mask {
  position: absolute;
  z-index: 5;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.25);
  display: flex;
  transition: opacity 0.4s ease;
}

.modal-container {
  width: v-bind("options.width ? options.width + 'px' : 'max-content'");
  min-width: 400px;
  max-width: min(560px, calc(100% - 32px));
  margin: auto;
  padding: 22px;
  background-color: var(--color-background);
  border-radius: 6px;
  box-shadow: var(--box-shadow);
  transition: transform 0.6s cubic-bezier(0.4, 2, 0.6, 1);
}

.modal-header h2 {
  margin-inline: 4px;
  color: var(--color-titlebar);
}

.modal-body {
  margin: 20px 4px;
  overflow-wrap: anywhere;
}

.modal-divider {
  width: 100%;
  height: 2px;
  background: var(--color-titlebar);
  margin: 8px 0;
  border: none;
  padding: 0;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.modal-input {
  width: 100%;
  font-size: 13px;
}

.modal-select {
  display: flex;
  flex-direction: column;
  gap: 2px;
  --list-min-height: 42px;
}

.modal-select :deep(.gameinfo-container) {
  color: var(--color-text-black);
  transition: color 0.2s ease;
}

.modal-select :deep(.gameinfo-container[aria-pressed='true']) {
  color: var(--color-tint);
}

/*
 * 对于 transition="modal" 的元素来说
 * 当通过 Vue.js 切换它们的可见性时
 * 以下样式会被自动应用。
 */

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-container {
  transform: rotate(-5deg) translateY(60px);
}

.modal-leave-to .modal-container {
  transform: rotate(5deg) translateY(60px);
}
</style>
