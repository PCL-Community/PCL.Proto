<script lang="ts" setup>

defineProps<INavItemGroup>()
import SideNavItem from './SideNavItem.vue';
import { type INavItemGroup } from '@/options/naviOptions';

</script>

<template>
    <div>
        <p class="side-group-title">{{ title }}</p>
        <div class="side-group-content">
            <SideNavItem v-for="item in content" :item-name="item.itemName" :icon="item.icon" :linkto="item.linkto" />
        </div>
    </div>
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