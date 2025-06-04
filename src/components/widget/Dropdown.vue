<script setup lang="ts">
import IconUnfold from '../icons/control/IconUnfold.vue';
import { type ISetupOption } from '@/util/setup';

defineProps<{
    options: ISetupOption[];
}>();

const model = defineModel<string>();
function updateModel(event: Event) {
    model.value = (event.target as HTMLSelectElement).value;
}

</script>

<template lang="pug">
    .dropdown-wrapper
        select(:value="model" @change="updateModel")
            option(v-for="option in options" 
                :key="option.value" 
                :value="option.value")
                | {{ option.text }}
        i: IconUnfold
</template>

<style lang="css" scoped>
.dropdown-wrapper {
    position: relative;
}

i {
    position: absolute;
    right: 0.5rem;
    top: 30%;
    transform: rotate(0deg);
    pointer-events: none;
    display: flex;
    place-items: center;
    color: var(--color-tint-lightist);
    transition: transform 0.4s ease;
}

select:focus+i {
    transform: rotate(180deg);
}

select {
    background-color: transparent;
    font-size: 12px;
    padding: 0.3rem 0.8rem;
    border-radius: 4px;
    border: 1px solid var(--color-tint-lightist);
    transition: background-color 0.3s ease;
    color: var(--color-text-black);
    appearance: none;
    -webkit-appearance: none;
    -moz-appearance: none;
    width: 100%;
}

select:hover {
    background-color: var(--color-tint-lighter);
}

select::-ms-expand {
    display: none;
}
</style>