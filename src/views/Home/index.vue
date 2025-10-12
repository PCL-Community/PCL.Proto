<script setup lang="ts">
import PButton from '@/components/widget/PButton.vue'
import useSideNavState from '@/stores/windowState'
import { computed, nextTick, onMounted, ref } from 'vue'
import MinecraftAvatar from '@/components/widget/MinecraftAvatar.vue'
import { useAccountInfo } from '@/stores/account'
import { useSelectedInstance } from '@/stores/gameLaunch'
import router from '@/router'
import { invoke } from '@tauri-apps/api/core'
import { useRepositoriesStore } from '@/stores/repositories'
import sideTip from '@/composables/sideTip'
import { animate, stagger } from 'motion-v'

const subviewRef = ref<HTMLElement>()
const sideNavState = useSideNavState()
const accontInfo = useAccountInfo()
const selectedInstance = useSelectedInstance()

onMounted(() => {
  sideNavState.setWidthOfPageDefault('home')
  nextTick(() => {
    animateSubview()
  })
  function animateSubview() {
    if (subviewRef.value) {
      const allChildren = Array.from(subviewRef.value.children)
      animate(
        allChildren,
        { y: [-20, 0], opacity: [0, 1] },
        {
          delay: stagger(0.06, { startDelay: 0 }),
          type: 'spring',
          duration: 0.6,
          bounce: 0.49,
        },
      )
    }
  }
})

const versionSelectClicked = async () => {
  await useRepositoriesStore().fetchFromBackend()
  router.push({ path: '/instance_select' })
}

const InstanceSettingClicked = () => {
  if (selectedInstance.instance_info) {
    router.push({ name: 'instance_setting' })
  } else {
    sideTip.show('Please select an instance first')
  }
}

const launchGame = () => {
  invoke('launch_game').catch((err) => {
    sideTip.show(`Failed to launch: ${err}`, 'warn')
  })
  console.log('[game] lanuch invoked')
}

const gameName = computed(() => {
  return selectedInstance.instance_info?.name || 'No Instance Selected'
})
</script>

<template lang="pug">
  .view-content
    aside.left
      #center
        //- MinecraftAvatar(type="url", src='default-skin/Steve_(classic_texture)_JE6.png')
        //- MinecraftAvatar(type='uuid', src='31bbe537-9fea-4e68-aa4a-d7aacca23d13')
        MinecraftAvatar(:skinUrl="accontInfo.skinUrl")
        input(v-model="accontInfo.username")
        p.gray 点击上方用户名可输入
      #button-grid
        PButton#launch(type="tint" :click="launchGame")
          p {{ $t('home.launch_game') }}
          p.gray {{ gameName }}
        PButton(:click="versionSelectClicked") {{ $t('home.instance_select') }}
        PButton(:click="InstanceSettingClicked") {{ $t('home.instance_setting') }}

    article.subview(ref="subviewRef")
      RouterView()

</template>

<style scoped>
#center > input {
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
  width: v-bind('sideNavState.sideNavWidthStr');

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
</style>
