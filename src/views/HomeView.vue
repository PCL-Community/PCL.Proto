<script setup lang="ts">
import MyButton from '@/components/widget/MyButton.vue';
import { sideNavState, defaultWidths, sideNavWidthStr } from '@/router/windowState';
import { animateCssFor } from '@/util/animateCSS';
import { nextTick, onMounted, onUnmounted, ref } from 'vue';
import { useRouter } from 'vue-router';

const subviewRef = ref<HTMLElement>()
const router = useRouter()
let removeRouteGuard: (() => void) | null = null

onMounted(() => {
  sideNavState.width = defaultWidths.home
  removeRouteGuard = router.afterEach(() => {
    nextTick(() => {
      animateSubview()
    })
  })
  nextTick(() => {
    animateSubview()
  })
  function animateSubview() {
    if (subviewRef.value) {
      const allChildren = subviewRef.value.children
      animateCssFor(allChildren, 'fadeInDown', 30)
    }
  }
})

onUnmounted(() => {
  removeRouteGuard?.()
})

</script>

<template lang="pug">
  .view-content
    aside.left
      | 主页
      MyButton()

    article.subview(ref="subviewRef")
      RouterView()

</template>

<style scoped>
aside {
  height: 100%;
  flex: 0 0 auto;
  padding: 20px;
}

aside.left {
  width: v-bind('sideNavWidthStr');
}

article {
  flex: 1 1 0;
  overflow-y: auto;
  padding-bottom: 76px;
  /*我也不知道为什么要加这个padding，反正不加就会被遮挡 */
}

.view-content {
  display: flex;
  width: 100%;
  height: 100%;
  box-sizing: border-box;
}
</style>