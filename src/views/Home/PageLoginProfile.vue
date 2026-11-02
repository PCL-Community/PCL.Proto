<script setup lang="ts">
import { computed } from 'vue'
import { useAccountInfo } from '@/stores/account'
import PListItem from '@/components/widget/PListItem.vue'
import PCard from '@/components/widget/PCard.vue'
import PHint from '@/components/widget/PHint.vue'
import PSkin from '@/components/widget/PSkin.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import type { ProfileView } from '@/types/profile'
import { pclText } from '@/api/pclLocale'
import { useModal } from '@/composables/useModal'
import { ModalWidthVariant } from '@/types/modal'
import { getProfileInfo } from '@/util/profile'
import { showUnavailable } from '@/util/showUnavailable'
import sideTip from '@/composables/sideTip'
const account = useAccountInfo()
const emit = defineEmits<{ view: [view: ProfileView] }>()
const text = pclText
const props = defineProps<{ requireMicrosoft?: boolean }>()
const modal = useModal()

const profileInfo = computed(() => getProfileInfo(account.profile))
const profileIsOffline = computed(() => (account.profile?.profileType ?? 'offline') === 'offline')

async function copyUuid() {
  const value = account.profile?.uuid
  if (!value) return
  try {
    await navigator.clipboard.writeText(value)
    sideTip.show(text('Proto.Clipboard.CopySuccess'), 'success')
  } catch {
    sideTip.show(text('Proto.Clipboard.CopyFailed'), 'warn')
  }
}

async function createProfile() {
  const selectOptions = [
    {
      value: 'microsoft',
      label: text('Launch.Account.Type.Microsoft'),
      icon: 'lucide/shield-check',
    },
    ...(props.requireMicrosoft
      ? []
      : [
          {
            value: 'auth',
            label: text('Launch.Account.Type.ThirdParty'),
            icon: 'lucide/network',
          },
          {
            value: 'offline',
            label: text('Launch.Account.Type.Offline'),
            icon: 'lucide/link-2-off',
          },
        ]),
  ]
  const result = await modal.open({
    title: text('Launch.Account.Profile.Create.SelectAuthType.Title'),
    width: ModalWidthVariant.Slim,
    selectOptions,
    confirmText: text('Common.Action.Continue'),
    cancelText: text('Common.Action.Cancel'),
  })
  if (!result.confirmed || !result.selected) return
  emit('view', result.selected as ProfileView)
}
</script>
<template>
  <section class="account-list">
    <PHint v-if="requireMicrosoft" severity="warning">{{
      text('Launch.Account.Profile.RequireMicrosoftHint')
    }}</PHint>
    <PListItem
      v-if="account.username"
      class="profile-item"
      :title="account.username"
      :subtitle="profileInfo"
      clickable
      @click="() => emit('view', 'skin')"
    >
      <template #icon>
        <div class="profile-head">
          <PSkin :skin-url="account.skinUrl" />
        </div>
      </template>
      <template #icon-actions>
        <button
          v-if="profileIsOffline"
          class="pcl-icon-button"
          type="button"
          :title="text('Launch.Account.Profile.ChangeUuid')"
          :aria-label="text('Launch.Account.Profile.ChangeUuid')"
          @click="showUnavailable"
        >
          <PSvgIcon name="lucide/pencil" :size="15" />
        </button>
        <button
          v-else
          class="pcl-icon-button"
          type="button"
          :title="text('Launch.Account.Profile.CopyUuid')"
          :aria-label="text('Launch.Account.Profile.CopyUuid')"
          @click="copyUuid"
        >
          <PSvgIcon name="lucide/copy" :size="15" />
        </button>
        <button
          class="pcl-icon-button"
          type="button"
          :title="text('Launch.Account.Profile.Delete')"
          :aria-label="text('Launch.Account.Profile.Delete')"
          @click="showUnavailable"
        >
          <PSvgIcon name="lucide/trash-2" :size="15" />
        </button>
      </template>
    </PListItem>
    <PCard hide-title class="create-card">
      <button
        type="button"
        class="pcl-icon-button create-profile"
        :title="text('Launch.Account.Profile.Create')"
        :aria-label="text('Launch.Account.Profile.Create')"
        @click="createProfile"
      >
        <PSvgIcon name="lucide/user-plus" :size="14" />
      </button>
    </PCard>
  </section>
</template>
<style scoped>
.account-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
}
.profile-item {
  align-self: center;
  width: min(100%, 220px);
  height: 42px;
  --list-title-size: 14px;
}
.profile-item :deep(.subtitle) {
  font-size: 12px;
}
.profile-item :deep(.list-actions) {
  margin-right: 5px;
}
.profile-head {
  position: relative;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  flex-shrink: 0;
}
.profile-head :deep(.skin) {
  position: absolute;
  left: 50%;
  top: 50%;
  margin: 0;
  transform: translate(-50%, -50%) scale(0.625);
}
.create-card {
  align-self: center;
  margin-top: auto;
  padding: 3px 9px 3px 10px;
}
.create-profile {
  width: 24px;
  height: 24px;
}
</style>
