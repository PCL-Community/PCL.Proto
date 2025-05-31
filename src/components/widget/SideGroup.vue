<script lang="ts" setup>

defineProps<INavItemGroup>()
import SideNavItem from './SideNavItem.vue';
import { type INavItemGroup } from '@/router/naviOptions';
</script>

<template lang="pug">
    div
        //- 添加 sidenav-line 类是为了方便外部做动画的
        p.side-group-title.sidenav-line {{title}}
        .side-group-content
            SideNavItem.sidenav-line(
                v-for="item in content" 
                :item-name="item.itemName" 
                :icon="item.icon" 
                :linkto="item.linkto")
</template>


<style lang="css" scoped>
.side-group-content {
    display: flex;
    flex-direction: column;
}

.side-group-title {
    margin-left: 12px;
    font-size: 11px;
    color: var(--color-text-grey);
}
</style>