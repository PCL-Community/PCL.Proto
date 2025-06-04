<script setup lang="ts">
import Dropdown from './Dropdown.vue';
import { type ISetupOption, type SetupItemType } from '@/util/setup';
import MyInput from './MyInput.vue';
import { computed } from 'vue';

defineProps<{
    label: string;
    options: ISetupOption[];
    type: SetupItemType;
}>();

const model = defineModel<string>();

</script>

<template lang="pug">
.setupitem-with-text-container
    p.text-label {{ label }}
    Dropdown.input(
        v-if="type == 'select' && options"
        :options="options"
        v-model="model"
    )
    MyInput.input(
        v-else-if="type == 'input'"
        :placeholder="options.find(v => v.key === 'placeholder')?.text"
        v-model="model"
    )
</template>

<style lang="css" scoped>
.setupitem-with-text-container {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-block: 4px;
}

.text-label {
    flex: 1;
    margin: 0;
}

.input {
    flex: 4;
}
</style>