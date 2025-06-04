<script lang="ts" setup>
type FoldStatus = 'unfold' | 'fold' | 'unfoldable';

const props = withDefaults(defineProps<{ hideTitle?: boolean, defaultFoldStatus?: FoldStatus }>(), {
    hideTitle: false,
    defaultFoldStatus: 'unfoldable'
})
import { ref } from 'vue';
import IconUnfold from '../icons/control/IconUnfold.vue';

const foldState = ref<FoldStatus>(props.defaultFoldStatus);
function SwitchFoldState() {
    switch (foldState.value) {
        case 'unfold':
            foldState.value = 'fold';
            break;
        case 'fold':
            foldState.value = 'unfold';
            break;
    }
}
</script>

<template lang="pug">
    .mycard(:class="foldState")
        .mycard-title(v-if="!hideTitle" @click="SwitchFoldState")
            p: slot(name="title") 标题
            i: IconUnfold()

        .mycard-content
            slot(name="content") 正文

</template>

<style>
.mycard:not(.unfoldable)>.mycard-title {
    cursor: pointer;
}

.mycard.unfoldable>.mycard-title>i {
    visibility: hidden;
}

.mycard {
    border-radius: 6px;
    background: var(--color-background-soft);
    box-shadow: 0px 0px 6px rgba(0, 0, 0, 0.1);
    padding: 8px 14px;
    flex-shrink: 0;
    display: grid;
    grid-template-rows: auto 1fr;
    transition: grid-template-rows 0.4s, box-shadow 0.4s;
}

.mycard.unfold {
    grid-template-rows: auto 1fr;
}

.mycard.fold {
    grid-template-rows: auto 0fr;
}

.mycard>.mycard-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    color: var(--color-text);
    height: fit-content;
}

.mycard>.mycard-title>i {
    color: var(--color-text);
    transition: transform 0.4s ease;
}

.mycard.unfold>.mycard-title>i {
    transform: rotate(180deg);
}

.mycard:hover {
    box-shadow: 0px 0px 6px var(--color-tint-shadow);
}

.mycard>.mycard-title>p *,
.mycard>.mycard-title>p {
    font-size: 12px;
    font-weight: bold;
    letter-spacing: 0px;
    line-height: 14.06px;
    text-align: left;
    vertical-align: top;
    transition: color 0.4s;
}

.mycard:hover>.mycard-title {
    color: var(--color-tint);
}

.mycard .mycard-content {
    font-size: 12px;
    margin: 9px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
}
</style>