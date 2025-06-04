<script setup lang="ts">
import SetupItem from '@/components/widget/SetupItem.vue';
import MyCard from '@/components/widget/MyCard.vue';
import { setup, setupOptions } from '@/util/setup';

</script>

<template lang="pug">
  MyCard()
    template(#title) 启动选项
    template(#content)

        SetupItem(:label="setupOptions.launch.defaultVersionIsolation.label"
                  :type="setupOptions.launch.defaultVersionIsolation.type"
                  :options="setupOptions.launch.defaultVersionIsolation.options"
                  v-model="setup.launch.defaultVersionIsolation")

        SetupItem(:label="setupOptions.launch.gameWindowTitle.label"
                  :type="setupOptions.launch.gameWindowTitle.type"
                  :options="setupOptions.launch.gameWindowTitle.options"
                  v-model="setup.launch.gameWindowTitle")
</template>