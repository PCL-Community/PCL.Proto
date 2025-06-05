<script lang="ts" setup>
type FoldStatus = 'unfold' | 'fold' | 'unfoldable';

const props = withDefaults(defineProps<{ hideTitle?: boolean, defaultFoldStatus?: FoldStatus }>(), {
    hideTitle: false,
    defaultFoldStatus: 'unfoldable'
})
import { nextTick, onMounted, ref } from 'vue';
import IconUnfold from '../icons/control/IconUnfold.vue';

const foldState = ref<FoldStatus>(props.defaultFoldStatus);

const cardHeight = ref<number>(40)

const mycardInnerRef = ref<HTMLElement>()

function SwitchFoldState() {
    switch (foldState.value) {
        case 'unfold':
            foldState.value = 'fold';
            cardHeight.value = 40
            break;
        case 'fold':
            foldState.value = 'unfold';
            nextTick(() => cardHeight.value = mycardInnerRef.value!.offsetHeight)
            break;
    }
}

onMounted(() => {
    nextTick(() => cardHeight.value = mycardInnerRef.value!.offsetHeight)
})

</script>

<template lang="pug">
.mycard-container(:class="foldState")
    .mycard(ref="mycardInnerRef")
        .mycard-title(v-if="!hideTitle" @click="SwitchFoldState")
            p: slot(name="title") 标题
            i: IconUnfold()
        Transition(name="card-content")
            .mycard-content(v-show="foldState == 'unfold' || foldState == 'unfoldable'")
                slot(name="content") 正文

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
    height: v-bind("cardHeight + 'px'");
    transition: height 0.4s cubic-bezier(.4, 1.4, .6, 1), box-shadow 0.4s;
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

.mycard-container.unfoldable .mycard-title>i {
    visibility: hidden;
}

.mycard {
    padding: 8px 14px;
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

.mycard-container.unfold .mycard-title>i {
    transform: rotate(180deg);
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

.mycard-container:hover>.mycard>.mycard-title {
    color: var(--color-tint);
}

.mycard>.mycard-content {
    font-size: 12px;
    margin: 9px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
}
</style>