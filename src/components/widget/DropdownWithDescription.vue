<script setup lang="ts">
import Dropdown from './Dropdown.vue';
import { type SetupOption } from '@/util/setup';

defineProps<{
    label: string;
    options: SetupOption[];
    modelValue: string;
}>();

defineEmits<{
    (e: 'update:modelValue', value: string): void;
}>();

</script>

<template lang="pug">
    .dropdown-with-text-container
        p.text-label {{ label}}
        Dropdown.dropdown-component(
            :options="options"
            :modelValue="modelValue"
            @update:modelValue="$emit('update:modelValue', $event)"
        )
</template>

<style lang="css" scoped>
.dropdown-with-text-container {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-block: 4px;
}

.text-label {
    flex: 1;
    margin: 0;
}

.dropdown-component {
    flex: 4;
}
</style>