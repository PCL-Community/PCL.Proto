<script setup lang="ts">
import PCard from './PCard.vue'
import PButton from './PButton.vue'
import PListItem from './PListItem.vue'
import PSvgIcon from './PSvgIcon.vue'
import sideTip from '@/composables/sideTip'
import { pclText } from '@/api/pclLocale'
const props = defineProps<{
  name: string
  address: string
  icon?: string
  motd?: string
  players?: string
  latency?: number
}>()
const emit = defineEmits<{ connect: []; refresh: []; edit: []; remove: [] }>()
async function copyAddress() {
  try {
    await navigator.clipboard.writeText(props.address)
    sideTip.show(pclText('Instance.Server.Card.AddressCopied', [props.address]), 'success')
  } catch {
    sideTip.show(pclText('Instance.Server.Card.CopyAddressFailed'), 'warn')
  }
}
const fallback = `${import.meta.env.BASE_URL}pcl/images/Icons/NoIcon.png`
</script>
<template>
  <PCard hide-title class="server-entry" overflow-visible
    ><PListItem
      :title="name"
      :subtitle="`${latency == null ? '—' : `${latency} ms`} · ${players || '—'}`"
      :icon="icon || fallback"
      ><template #actions
        ><p class="motd">{{ motd }}</p></template
      >
      <template #icon-actions>
        <button
          class="pcl-icon-button"
          type="button"
          @click="() => emit('connect')"
          :title="name"
          :aria-label="name"
        >
          <PSvgIcon name="lucide/play" :size="15" />
        </button>
        <details>
          <summary
            class="pcl-icon-button"
            :title="pclText('Instance.Server.Card.Edit')"
            :aria-label="pclText('Instance.Server.Card.Edit')"
          >
            <PSvgIcon name="lucide/settings" :size="15" />
          </summary>
          <div class="server-menu">
            <PButton inline @click="() => emit('refresh')">{{
              pclText('Common.Action.Refresh')
            }}</PButton
            ><PButton inline @click="copyAddress">{{
              pclText('Instance.Server.Card.CopyAddress')
            }}</PButton
            ><PButton inline @click="() => emit('edit')">{{
              pclText('Instance.Server.Card.Edit')
            }}</PButton
            ><PButton inline type="warn" @click="() => emit('remove')">{{
              pclText('Instance.Server.Card.Remove')
            }}</PButton>
          </div>
        </details></template
      ></PListItem
    ></PCard
  >
</template>
<style scoped>
.server-entry {
  margin: 8px 0;
  --list-min-height: 44px;
  --list-icon-size: 34px;
  --list-icon-radius: 6px;
}
.motd {
  font-family: var(--motd-font);
  width: min(350px, 30vw);
  white-space: pre-line;
  font-size: 12px;
}
details {
  position: relative;
}
summary {
  list-style: none;
}
summary::-webkit-details-marker {
  display: none;
}
.server-menu {
  position: absolute;
  z-index: 3;
  right: 0;
  top: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  min-width: 130px;
  padding: 6px;
  background: var(--color-background-soft);
  border-radius: 6px;
  box-shadow: var(--box-shadow);
}
.server-menu > a {
  padding: 6px;
  text-align: center;
  font-size: 13px;
}
</style>
