<script lang="ts">
import SideNavLayout from '@/layout/SideNavLayout.vue';
import { h } from 'vue';

export default {
    setup(props, ctx) {
        return () => h(SideNavLayout, {
            sideNavGroups: [{
                title: '启动',
                content: []
            }
            ]
        })
    },
}
</script>