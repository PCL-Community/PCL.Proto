<!-- src/components/GlobalModal.vue -->
<script setup lang="ts">
import { useModal } from '@/composables/modalService'

const { isOpen, content, title, close } = useModal()
</script>

<template>
    <Transition name="modal">
        <div v-if="isOpen" class="modal-mask">
            <div class="modal-container">
                <div class="modal-header">
                    <h3>{{ title }}</h3>
                </div>
                <div class="modal-body">
                    {{ content }}
                </div>
                <div class="modal-footer">
                    <button @click="close(false)">取消</button>
                    <button @click="close(true)">确认</button>
                </div>
            </div>
        </div>
    </Transition>
</template>

<style>
.modal-mask {
    position: fixed;
    z-index: 9998;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    transition: opacity 0.3s ease;
}

.modal-container {
    width: 300px;
    margin: auto;
    padding: 20px 30px;
    background-color: #fff;
    border-radius: 2px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.33);
    transition: all 0.3s ease;
}

.modal-header h3 {
    margin-top: 0;
    color: #42b983;
}

.modal-body {
    margin: 20px 0;
}

.modal-default-button {
    float: right;
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

.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
    -webkit-transform: scale(1.1);
    transform: scale(1.1);
}
</style>