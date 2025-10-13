<script setup lang="ts">
import { useModal } from '@/composables/useModal'
import { ref, watch } from 'vue'

const picked = ref('auto')
const checked = ref(false)

watch(checked, async (value) => {
  if (value == true) {
    let result = await useModal().open({
      title: '提醒',
      content: '内存优化巴拉巴拉……',
    })
    if (result.confirmed == false) {
      checked.value = false
    }
  }
})
</script>

<template>
  <div class="option">
    <input class="checkable" type="radio" id="one" value="auto" v-model="picked" />
    <label for="one">自动配置</label>
  </div>
  <div class="option">
    <input class="checkable" type="radio" id="two" value="custom" v-model="picked" />
    <label for="two">自定义</label>
  </div>
  <div class="option">
    <input type="checkbox" id="checkbox" class="checkable" v-model="checked" />
    <label for="checkbox">启动游戏前进行内存优化</label>
  </div>
</template>
