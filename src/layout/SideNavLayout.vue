<script lang="ts">
import { defineComponent, onMounted, onUnmounted, useTemplateRef } from 'vue'
import useSideNavState from '@/stores/windowState'
import SideGroup from '@/components/widget/SideGroup.vue'
import { type INavItemGroup } from '@/types/naviOptions'
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
  setup(_props, context) {
    context.expose({ animateSubview })
    let observer: ResizeObserver | null = null
    const asideRef = useTemplateRef<HTMLElement>('asideRef')
    const subviewRef = useTemplateRef<HTMLElement>('subviewRef')
    const sideNavState = useSideNavState()

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
    })

    onUnmounted(() => {
      observer?.disconnect()
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
    article.subview(ref="subviewRef" v-card-drop-children-animate)
        RouterView(@animate-subview="animateSubview" @vue:updated="animateSubview")
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

@keyframes fade-in-left {
  70% {
    transform: translateX(2px);
  }
  100% {
    opacity: 1;
    transform: translateX(0);
  }
}

aside :deep(.sidenav-line) {
  opacity: 0;
  will-change: transform, opacity;
  transform: translateX(-100%);
  animation: fade-in-left 0.45s ease-in-out forwards;
}
</style>
