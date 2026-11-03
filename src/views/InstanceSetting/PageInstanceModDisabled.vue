<script setup lang="ts">
import { useRouter } from 'vue-router'
import { pclText } from '@/api/pclLocale'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'

const router = useRouter()
const text = pclText
</script>
<template>
  <div class="pcl-page">
    <PCard class="pcl-empty-state-card" hide-title>
      <div class="pcl-empty-state">
        <h2 class="pcl-empty-state-title pcl-text">
          {{ text('Instance.Resource.Mod.Disabled.Title') }}
        </h2>
        <div class="pcl-divider" />
        <p class="pcl-empty-state-message pcl-text">
          {{ text('Instance.Resource.Mod.Disabled.Message') }}
        </p>
        <div class="pcl-empty-state-actions">
          <PButton @click="() => router.push('/download/game')" type="tint">{{
            text('Instance.Resource.Mod.Disabled.GoDownload')
          }}</PButton>
          <PButton @click="() => router.push('/instance_select')">{{
            text('Instance.Resource.Mod.Disabled.SelectInstance')
          }}</PButton>
        </div>
      </div>
    </PCard>
  </div>
</template>
