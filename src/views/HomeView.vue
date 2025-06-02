<script setup lang="ts">
import MyButton from '@/components/widget/MyButton.vue';
import { sideNavState, defaultWidths, sideNavWidthStr } from '@/util/windowState';
import { animateCss, animateCssFor } from '@/util/animateCSS';
import { nextTick, onMounted, ref } from 'vue';

const subviewRef = ref<HTMLElement>()
const asideRef = ref<HTMLElement>()

onMounted(() => {
  sideNavState.width = defaultWidths.home
  nextTick(() => {
    animateCss(asideRef.value!, 'zoomIn')
    animateSubview()
  })
  function animateSubview() {
    if (subviewRef.value) {
      const allChildren = subviewRef.value.children
      animateCssFor(allChildren, 'fadeInDown', 30)
    }
  }
})

</script>

<template lang="pug">
  .view-content
    aside.left(ref="asideRef")
      #center
        p Text Center

      #button-grid
        MyButton#launch(type="tint")
          p 启动游戏
          p.gray Fabulouly Optimized
        MyButton 版本选择
        MyButton 版本设置

    article.subview(ref="subviewRef")
      RouterView()

</template>

<style scoped>
#button-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

#launch {
  grid-column: span 2;
}

#launch p.gray {
  color: var(--color-text-gray);
  font-size: 11px;
}

aside.left {
  height: 100%;
  flex: 0 0 auto;
  padding: 20px;
  /* padding-bottom: 68px; */
  width: v-bind('sideNavWidthStr');

  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: stretch;
}

#center {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1 1 auto;
}

article {
  flex: 1 1 0;
  overflow-y: auto;
}

.view-content {
  display: flex;
  width: 100%;
  height: 100%;
  box-sizing: border-box;
}
</style>