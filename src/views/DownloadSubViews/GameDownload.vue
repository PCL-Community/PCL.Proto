<script lang="ts" setup>
import CardInfoItem from '@/components/widget/CardInfoItem.vue';
import MyCard from '@/components/widget/MyCard.vue';
import { showIconPath } from '@/util/gameInfo';
import { getMinecraftVersions, versionData } from '@/api/gameVersions';
import { onMounted } from 'vue';
import sideTip from '@/composables/sideTip';
onMounted(async () => {
    if (!versionData.value) {
        sideTip.show('正在获取游戏版本信息...')
        versionData.value = await getMinecraftVersions()
    }
})
</script>

<template lang="pug">
    MyCard(:hide-title="false")
        template(#title) 最新版本
        template(#content)
            CardInfoItem(:icon="showIconPath.grass" :title="versionData?.latest.release.id" :subtitle="`最新正式版，发布于 ${versionData?.latest.release.releaseTime}`")
            CardInfoItem(:icon='showIconPath.command' :title="versionData?.latest.snapshot.id" :subtitle="`最新预览版，发布于 ${versionData?.latest.snapshot.releaseTime}`")
    MyCard(default-fold-status="fold")
        template(#title) 正式版({{ versionData?.release.length }})
        template(#content)
            CardInfoItem(v-for="item in versionData?.release" :key="item.id" :icon="showIconPath.grass" :title="item.id" :subtitle="`发布于 ${item.releaseTime}`")
    MyCard(default-fold-status="fold")
        template(#title) 预览版({{ versionData?.snapshot.length }})
        template(#content)
            CardInfoItem(v-for="item in versionData?.snapshot" :key="item.id" :icon="showIconPath.command" :title="item.id" :subtitle="`发布于 ${item.releaseTime}`")
    MyCard(default-fold-status="fold")
        template(#title) 远古版({{ versionData?.old.length }})
        template(#content)
            CardInfoItem(v-for="item in versionData?.old" :key="item.id" :icon="showIconPath.stone" :title="item.id" :subtitle="`发布于 ${item.releaseTime}`")
    MyCard(default-fold-status="fold")
        template(#title) 愚人节版(尚未分类)

</template>
