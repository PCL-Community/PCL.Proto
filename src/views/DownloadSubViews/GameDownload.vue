<script lang="ts" setup>
import CardInfoItem from '@/components/widget/CardInfoItem.vue';
import MyCard from '@/components/widget/MyCard.vue';
import { gameInfoRecord } from '@/util/gameInfo';
</script>

<template lang="pug">
    MyCard(:hide-title="false")
        template(#title) 最新版本
        template(#content)
            CardInfoItem(:icon='gameInfoRecord.command' title="25w20a" subtitle="最新预览版，发布于 2025/05/13 19:46")
            CardInfoItem(:icon='gameInfoRecord.grass' title="1.20.4" subtitle="阿巴巴巴巴")
            

</template>
