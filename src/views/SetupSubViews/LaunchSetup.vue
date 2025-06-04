<script setup lang="ts">
import Dropdown from '@/components/widget/Dropdown.vue';
import DropdownWithDescription from '@/components/widget/DropdownWithDescription.vue';
import MyButton from '@/components/widget/MyButton.vue';
import MyCard from '@/components/widget/MyCard.vue';
import { setup, setupOptions } from '@/util/setup';
</script>

<template lang="pug">
    MyCard()
        template(#title) 启动选项
        template(#content)
            DropdownWithDescription(
                v-for="(field, key) in setupOptions.launch"
                :key="key"
                :label="field.text"
                :options="field.options"
                v-model="setup.launch[key]"
            )
    MyCard()
        template(#title) 游戏内存
        template(#content)
            DropdownWithDescription(
                v-for="(field, key) in setupOptions.gameMemory"
                :key="key"
                :label="field.text"
                :options="field.options"
                v-model="setup.gameMemory[key]"
            )
</template>