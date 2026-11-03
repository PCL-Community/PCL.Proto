<script setup lang="ts">
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'

const text = pclText
</script>
<template>
  <div class="pcl-page">
    <div style="display: flex; flex-direction: column">
      <PCard :title="text('Setup.Log.Operations')">
        <div
          style="
            margin: 0px 20px 20px 25px;
            display: flex;
            flex-direction: row;
            align-items: center;
            flex-wrap: wrap;
          "
        >
          <PButton
            style="min-width: 100px; margin: 0px 15px 0px 0px"
            @click="showUnavailable"
            type="tint"
            >{{ text('Setup.Log.Export') }}</PButton
          >
          <PButton style="min-width: 120px; margin: 0px 15px 0px 0px" @click="showUnavailable">{{
            text('Setup.Log.ExportAll')
          }}</PButton>
          <PButton style="min-width: 120px; margin: 0px 15px 0px 0px" @click="showUnavailable">{{
            text('Setup.Log.OpenDirectory')
          }}</PButton>
          <PButton style="min-width: 120px" @click="showUnavailable" type="warn">{{
            text('Setup.Log.ClearHistory')
          }}</PButton>
        </div>
      </PCard>
      <PCard :title="text('Setup.Log.AllLogs')">
        <div style="margin: 0px 18px 18px 20px; display: flex; flex-direction: column">
          <p class="pcl-empty">{{ pclText('Proto.Empty.LogFiles') }}</p>
        </div>
      </PCard>
    </div>
  </div>
</template>
