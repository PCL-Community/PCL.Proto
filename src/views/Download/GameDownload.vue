<script lang="ts" setup>
import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import { showIconPath } from '@/util/gameInfo'
import { getMinecraftVersions } from '@/api/gameVersions'
import { onMounted } from 'vue'
let { cacheVersionData, versionDataRef } = getMinecraftVersions()

onMounted(() => {
  if (!cacheVersionData) {
    console.warn('缓存不存在')
  }
})
</script>

<template lang="pug">
    PCard(:hide-title="false" v-if="versionDataRef")
        template(#title) 最新版本
        template(#content)
            CardInfoItem(:icon="showIconPath.grass" :title="versionDataRef?.latest.release.id" :subtitle="`最新正式版，发布于 ${versionDataRef?.latest.release.releaseTime}`" isGameInfo) 
            CardInfoItem(:icon='showIconPath.command' :title="versionDataRef?.latest.snapshot.id" :subtitle="`最新预览版，发布于 ${versionDataRef?.latest.snapshot.releaseTime}`" isGameInfo)
    PCard(default-fold-status="fold")
        template(#title) 正式版({{ versionDataRef?.release.length }})
        template(#content)
            CardInfoItem(v-for="item in versionDataRef?.release" :key="item.id" :icon="showIconPath.grass" :title="item.id" :subtitle="`发布于 ${item.releaseTime}`")
    PCard(default-fold-status="fold")
        template(#title) 预览版({{ versionDataRef?.snapshot.length }})
        template(#content)
            CardInfoItem(v-for="item in versionDataRef?.snapshot" :key="item.id" :icon="showIconPath.command" :title="item.id" :subtitle="`发布于 ${item.releaseTime}`")
    PCard(default-fold-status="fold")
        template(#title) 远古版({{ versionDataRef?.old.length }})
        template(#content)
            CardInfoItem(v-for="item in versionDataRef?.old" :key="item.id" :icon="showIconPath.stone" :title="item.id" :subtitle="`发布于 ${item.releaseTime}`")
    PCard(default-fold-status="fold")
        template(#title) 愚人节版(尚未分类)

</template>
