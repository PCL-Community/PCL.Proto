<script setup lang="ts">
import MyButton from '@/components/widget/MyButton.vue';
import { sideNavState, defaultWidths, sideNavWidthStr } from '@/util/windowState';
import { animateCssFor } from '@/util/animateCSS';
import { nextTick, onMounted, ref } from 'vue';
import MinecraftAvatar from '@/components/widget/MinecraftAvatar.vue';
import { accontInfo } from '@/util/account';
import { selectedGame } from '@/util/gameLaunch';

const subviewRef = ref<HTMLElement>()

onMounted(() => {
  sideNavState.width = defaultWidths.home
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

</script>

<template lang="pug">
  .view-content
    aside.left
      #center
        //- MinecraftAvatar(type="url", src='default-skin/Steve_(classic_texture)_JE6.png')
        //- MinecraftAvatar(type='uuid', src='31bbe537-9fea-4e68-aa4a-d7aacca23d13')
        MinecraftAvatar(type='username' :src='accontInfo.username')
        input(v-model="accontInfo.username")
        p.gray 点击上方用户名可输入
      #button-grid
        MyButton#launch(type="tint")
          p 启动游戏
          p.gray {{ selectedGame.name }}
        MyButton 版本选择
        MyButton 版本设置

    article.subview(ref="subviewRef")
      RouterView()

</template>

<style scoped>
#center>input {
  font-size: 15px;
  text-align: center;
  color: var(--color-text-black);
  font-family: 'PCL-English';
  border: none;
  border-radius: 4px;
  outline: none;
  background-color: transparent;
}

#button-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

#launch {
  grid-column: span 2;
}

p.gray {
  color: var(--color-text-grey);
  font-size: 11px;
}

@keyframes pclZoomIn {
  from {
    opacity: 0;
    transform: scale(0.9);
  }

  to {
    opacity: 1;
    transform: scale(1);
  }
}

aside.left {
  height: 100%;
  flex: 0 0 auto;
  padding: 20px;
  width: v-bind('sideNavWidthStr');

  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: stretch;

  animation: pclZoomIn 0.4s ease forwards;
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