import { ref } from "vue";
import type { SideTipType, ISideTipItem } from "@/types/sideTip";
const tips = ref<ISideTipItem[]>([])
let uid = 0

export function useSideTip() {
    const open = (msg: string, type: SideTipType = 'default') => {
        const id = ++uid
        const item: ISideTipItem = { id, message: msg, colorType: type }
        tips.value.push(item)
        setTimeout(() => {
            tips.value = tips.value.filter(t => t.id !== id)
        }, 2000)
    }

    return {
        tips,
        open
    }
}