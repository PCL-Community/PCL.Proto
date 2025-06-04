<script setup lang="ts">
import Dropdown from './Dropdown.vue';
defineProps<{
    label: string;
    options: string[];
}>();

</script>

<template lang="pug">
    .dropdown-with-text-container
        p.text-label {{ options}}
        Dropdown.dropdown-component()
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