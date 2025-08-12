<script lang="ts">
import { defineComponent, onMounted, onUnmounted, useTemplateRef } from 'vue'
import useSideNavState from '@/stores/windowState'
import { nextTick } from 'vue'
import SideGroup from '@/components/widget/SideGroup.vue'
import { type INavItemGroup } from '@/types/naviOptions'
import { animateCssFor } from '@/util/animateCSS'
import { useRouter } from 'vue-router'
import PLoading from '@/components/widget/PLoading.vue'

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
  setup() {
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
        // TODO)) 排除loading组件
        const allChildren = subviewRef.value.children
        animateCssFor(allChildren, 'fadeInDown', 30)
      }
    }

    function animateSidenavLines() {
      if (asideRef.value) {
        const sidenavLines = asideRef.value.querySelectorAll('.sidenav-line')
        animateCssFor(sidenavLines, 'fadeInLeft', 20)
      } else {
        console.warn('[nav] asideRef is null')
      }
    }

    onMounted(async () => {
      console.log('[nav] SideNavLayout mounted')
      observer = new ResizeObserver(updateAsideBackgroundWidth)
      observer.observe(asideRef.value!)
      removeRouteGuard = router.afterEach((to, from) => {
        // console.log('[nav] afterEach', to, from)
        nextTick(() => {
          animateSubview()
          // 均使用本组件的页面切换时本组件会复用，因此需要重新应用侧边动画
          // 但是在同样一级页面内跳转二级页面时无需再次动画
          if (from.matched[0].name !== to.matched[0].name) {
            animateSidenavLines()
          }
        })
      })
      nextTick(() => {
        animateSidenavLines()
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
    }
  },
})
</script>

<template lang="pug">
.view-content
    aside(ref="asideRef")
        SideGroup(
            v-for="group in sideNavGroups"
            :title="group.title"
            :content="group.content"
)
    article.subview(ref="subviewRef")
        RouterView()
</template>

<style scoped>
.view-content {
  display: flex;
  width: 100%;
  height: 100%;
  box-sizing: border-box;
}

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
</style>
