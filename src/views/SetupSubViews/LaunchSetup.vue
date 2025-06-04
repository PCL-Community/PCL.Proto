<script setup lang="ts">
import Dropdown from '@/components/widget/Dropdown.vue';
import DropdownWithDescription from '@/components/widget/DropdownWithDescription.vue';
import MyButton from '@/components/widget/MyButton.vue';
import MyCard from '@/components/widget/MyCard.vue';

</script>

<template lang="pug">
    MyCard()
        template(#title) 启动选项
        template(#content)
            DropdownWithDescription()
</template>