<script setup lang="ts">
import { useModal } from '@/composables/useModal'
import { ref } from 'vue';
import MyButton from './widget/MyButton.vue';

const { isOpen, options, close } = useModal()
</script>

<template lang="pug">
  Transition(name="modal")
    .modal-mask(v-if="isOpen")
      .modal-container
        .modal-header
          h2 {{ options.title ?? '模态框标题未设置' }}

        .modal-divider

        .modal-body {{ options.content ?? '模态框正文未设置' }}

        .modal-footer
          MyButton(@click="close(true)" type="tint") 确认
          MyButton(@click="close(false)") 取消

</template>

<style>
.modal-mask {
  position: absolute;
  z-index: 2;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.25);
  display: flex;
  transition: opacity 0.4s ease;
}

.modal-container {
  width: v-bind("(options.width ?? '400') + 'px'");
  margin: auto;
  padding: 22px;
  background-color: var(--color-background);
  border-radius: 6px;
  box-shadow: var(--box-shadow);
  transition: transform 0.6s cubic-bezier(.4, 2, .6, 1);
}

.modal-header h2 {
  margin-inline: 4px;
  color: var(--color-titlebar);
}

.modal-body {
  margin: 20px 4px;
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

/*
 * 对于 transition="modal" 的元素来说
 * 当通过 Vue.js 切换它们的可见性时
 * 以下样式会被自动应用。
 */

.modal-enter-from {
  opacity: 0;
}

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