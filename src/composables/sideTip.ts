import { ref } from 'vue'
import type { SideTipType, ISideTipItem } from '@/types/sideTip'
const tips = ref<ISideTipItem[]>([])
let uid = 0

// 全局消息通知的单例对象
const sideTip = {
  show: (msg: string, type: SideTipType = 'default') => {
    const id = ++uid
    const item: ISideTipItem = { id, message: msg, colorType: type }
    tips.value.push(item)
    setTimeout(() => {
      tips.value.shift()
    }, 2000)
  },
  tips,
}

export default sideTip
