<script lang="ts">
import { defineComponent, nextTick, onMounted, onUnmounted, useTemplateRef } from 'vue'
import useSideNavState from '@/stores/windowState'
import SideGroup from '@/components/widget/SideGroup.vue'
import { type INavItemGroup } from '@/types/naviOptions'
import { useRouter } from 'vue-router'
import PLoading from '@/components/widget/PLoading.vue'
import cardDropAnimate from '@/util/cardDropAnimate'

export default defineComponent({
  name: 'SideNavLayout',
  components: {
    SideGroup,
    PLoading,
  },
  props: {
    sideNavGroups: {
      type: Array as () => INavItemGroup[],
      required: true,
    },
  },
  setup(props, context) {
    context.expose({ animateSubview })
    let observer: ResizeObserver | null = null
    const asideRef = useTemplateRef<HTMLElement>('asideRef')
    const subviewRef = useTemplateRef<HTMLElement>('subviewRef')
    const router = useRouter()
    const sideNavState = useSideNavState()

    let removeRouteGuard: (() => void) | null = null

    function updateAsideBackgroundWidth() {
      if (asideRef.value) {
        sideNavState.setWidth(asideRef.value.offsetWidth)
      }
    }

    function animateSubview() {
      if (subviewRef.value) {
        const allChildren = Array.from(subviewRef.value.children)
        cardDropAnimate(allChildren)
      }
    }

    onMounted(async () => {
      console.log('[nav] SideNavLayout mounted')
      asideRef.value?.querySelectorAll('.sidenav-line').forEach((el_, i) => {
        let el = el_ as HTMLDivElement
        el.style.animationPlayState = 'paused'
        el.style.animationDelay = `${i * 0.02}s`
        requestAnimationFrame(() => {
          el.style.animationPlayState = 'running'
        })
      })
      observer = new ResizeObserver(updateAsideBackgroundWidth)
      observer.observe(asideRef.value!)
      removeRouteGuard = router.afterEach((to, from) => {
        nextTick(() => {
          animateSubview()
        })
      })
      nextTick(() => {
        animateSubview()
      })
    })

    onUnmounted(() => {
      observer?.disconnect()
      removeRouteGuard?.()
    })

    return {
      asideRef,
      subviewRef,
      animateSubview,
    }
  },
})
</script>

<template lang="pug">
.view-content
    aside(ref="asideRef")
        SideGroup(
            v-for="group in sideNavGroups"
            v-bind="group"
        )
    article.subview(ref="subviewRef")
        RouterView(@animate-subview="animateSubview")
</template>

<style scoped>
article {
  flex: 1 1 auto;
  overflow-y: auto;
}

aside {
  padding: 14px 0 0 0;
  height: 100%;
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  gap: 28px;
}

@keyframes fadeInLeft {
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

aside :deep(.sidenav-line) {
  opacity: 0;
  will-change: transform, opacity;
  transform: translateX(-100%);
  animation: fadeInLeft 0.4s ease forwards;
}
</style>
