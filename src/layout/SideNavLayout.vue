<script lang="ts">
import { defineComponent, onMounted, onUnmounted } from 'vue'
import { sideNavState } from '@/router/windowState'
import { nextTick, ref } from 'vue'
import SideGroup from '@/components/widget/SideGroup.vue'
import { type INavItemGroup } from '@/router/naviOptions'
import { animateCssFor } from '@/animateCSS'
import { useRouter } from 'vue-router'

export default defineComponent({
    name: 'SideNavLayout',
    components: {
        SideGroup
    },
    props: {
        sideNavGroups: {
            type: Array as () => INavItemGroup[],
            required: true
        }
    },
    setup() {
        let observer: ResizeObserver | null = null
        const asideRef = ref<HTMLElement>()
        const subviewRef = ref<HTMLElement>()
        const router = useRouter()
        let removeRouteGuard: (() => void) | null = null

        function updateAsideBackgroundWidth() {
            if (asideRef.value) {
                sideNavState.width = asideRef.value.offsetWidth
            }
        }

        function animateSubviews() {
            if (subviewRef.value) {
                const allChildren = subviewRef.value.children
                animateCssFor(allChildren, 'fadeInDown', 30)
            }
        }

        function animateSidenavLines() {
            if (asideRef.value) {
                const sidenavLines = asideRef.value.querySelectorAll('.sidenav-line')
                animateCssFor(sidenavLines, 'fadeInLeft', 20)
            }
        }

        onMounted(async () => {
            observer = new ResizeObserver(updateAsideBackgroundWidth)
            observer.observe(asideRef.value!)
            removeRouteGuard = router.afterEach(() => {
                nextTick(() => {
                    animateSubviews()
                })
            })
            nextTick(() => {
                animateSidenavLines()
                animateSubviews()
            })
        })

        onUnmounted(() => {
            observer?.disconnect()
            removeRouteGuard?.()
        })

        return {
            asideRef,
            subviewRef
        }
    }
})
</script>

<template>
    <div class="view-content">
        <aside ref="asideRef">
            <SideGroup v-for="group in sideNavGroups" :title="group.title" :content="group.content" />
        </aside>
        <article class="subview" ref="subviewRef">
            <RouterView />
        </article>
    </div>
</template>
<!-- <template lang="pug">
    .view-content
        aside 
</template> -->

<style scoped>
.view-content {
    display: flex;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
}

article {
    flex: 1 1 0;
    overflow-y: auto;
    padding-bottom: 76px;
}

aside {
    padding: 14px 0 0 0;
    height: 100%;
    flex: 0 0 auto;
    display: flex;
    flex-direction: column;
    gap: 28px;
}
</style>