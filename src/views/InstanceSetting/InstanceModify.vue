<script setup lang="ts">
import OverviewCard from './OverviewCard'
import { type pluginType, pluginShowText } from '@/util/gameInfo'
import ModifyCard from '@/components/widget/ModifyCard.vue'

const pluginTypes = Object.keys(pluginShowText) as pluginType[]
</script>

<template>
  <OverviewCard />
  <ModifyCard v-for="type in pluginTypes" :key="type" :plugin />
</template>
