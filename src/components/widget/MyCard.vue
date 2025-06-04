<script lang="ts" setup>
type FoldStatus = 'unfold' | 'fold' | 'unfoldable';

const props = defineProps<{ hideTitle?: boolean, defaultFoldState?: FoldStatus }>()
import { watch } from 'vue';
import IconUnfold from '../icons/control/IconUnfold.vue';

function SwitchFoldState() {

}

</script>

<template lang="pug">
    .mycard(:class="(defaultFoldState ?? 'unfoldable')")
        .mycard-title(v-if="!hideTitle" @click="SwitchFoldState")
            p: slot(name="title") 标题
            i: IconUnfold()

        .mycard-content
            slot(name="content") 正文

</template>

<style>
.mycard {
    border-radius: 6px;
    background: var(--color-background-soft);
    box-shadow: 0px 0px 6px rgba(0, 0, 0, 0.1);
    padding: 8px 14px;
    transition: box-shadow 0.4s;
}

.mycard>.mycard-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    color: var(--color-text);
}

.mycard:hover {
    box-shadow: 0px 0px 6px var(--color-tint-shadow);
}

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
}
</style>