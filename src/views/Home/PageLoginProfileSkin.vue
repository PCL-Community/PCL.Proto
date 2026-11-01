<script setup lang="ts">
import { showUnavailable } from '@/util/showUnavailable'
import PSkin from '@/components/widget/PSkin.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import { useAccountInfo } from '@/stores/account'
import type { ProfileView } from '@/types/profile'
import { pclText } from '@/api/pclLocale'
import PCard from '@/components/widget/PCard.vue'
import { popupNativeMenu, type NativeMenuItem } from '@/util/nativeMenu'
import { getProfileTypeLabel } from '@/util/profile'
import { computed } from 'vue'

const account = useAccountInfo()
const emit = defineEmits<{ view: [view: ProfileView] }>()
const text = pclText
const profileTypeLabel = computed(() => getProfileTypeLabel(account.profile))
const skinActions = ['ChangeSkin', 'SaveSkin', 'RefreshSkin', 'ChangeCape'] as const
const editActions = ['EditPassword', 'EditUsername'] as const

function runAction(action: string) {
  if (action === 'EditUsername') emit('view', 'offline')
  else void showUnavailable()
}

async function openMenu(event: MouseEvent, actions: readonly string[]) {
  const items: NativeMenuItem[] = actions.map((action) => ({
    text: text(`Launch.Account.ProfileSkin.${action}`),
    action: () => runAction(action),
  }))
  try {
    await popupNativeMenu(event.currentTarget as Element, items)
  } catch (error) {
    console.error('Failed to open native menu', error)
  }
}
</script>
<template>
  <div class="profile-skin">
    <PSkin class="profile-avatar" :skin-url="account.skinUrl" /><strong>{{
      account.username || 'Player'
    }}</strong
    ><span>{{ profileTypeLabel }}</span>
    <div class="profile-actions-reveal">
      <PCard hide-title class="profile-actions-card"
        ><div class="profile-actions">
          <button
            type="button"
            class="pcl-icon-button profile-action"
            :aria-label="text('Launch.Account.ProfileSkin.SkinsAndCapes')"
            :title="text('Launch.Account.ProfileSkin.SkinsAndCapes')"
            @click="openMenu($event, skinActions)"
          >
            <PSvgIcon name="lucide/shirt" :size="14" />
          </button>
          <button
            type="button"
            class="pcl-icon-button profile-action"
            :aria-label="text('Launch.Account.ProfileSkin.EditProfile')"
            :title="text('Launch.Account.ProfileSkin.EditProfile')"
            @click="openMenu($event, editActions)"
          >
            <PSvgIcon name="lucide/pencil" :size="14" />
          </button>
          <button
            type="button"
            class="pcl-icon-button profile-action"
            @click="emit('view', 'profiles')"
            :aria-label="text('Launch.Account.ProfileSkin.SwitchProfile')"
            :title="text('Launch.Account.ProfileSkin.SwitchProfile')"
          >
            <PSvgIcon name="lucide/arrow-right-left" :size="14" />
          </button></div
      ></PCard>
    </div>
  </div>
</template>
<style scoped>
.profile-skin {
  position: relative;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  transform: translateY(10px);
}
.profile-skin::before {
  content: '';
  position: absolute;
  inset: -16px 0 0;
}
.profile-avatar {
  margin: 0;
}
.profile-skin strong {
  font-size: 17px;
  font-weight: normal;
}
.profile-skin > span {
  color: var(--color-text-grey);
  font-size: 12px;
}
.profile-actions {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 3px 9px 3px 10px;
  position: relative;
}
.profile-actions-reveal {
  position: absolute;
  top: 100%;
  width: 100%;
  display: flex;
  justify-content: center;
  padding-top: 8px;
  opacity: 0;
  /* CE PageLoginProfileSkin.ShowPanel/HidePanel: 120 ms linear fade. */
  transition: opacity 120ms linear;
}
.profile-skin:hover .profile-actions-reveal,
.profile-skin:focus-within .profile-actions-reveal {
  opacity: 1;
}
.profile-actions-card {
  border-radius: 5px;
}
.profile-action {
  width: 24px;
  height: 24px;
}
</style>
