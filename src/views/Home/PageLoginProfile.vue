<script setup lang="ts">
import { useAccountInfo, toAuthFailure, type McProfile, type UuidMode } from '@/stores/account'
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
import { dialog } from '@/util/loginErrors'
import sideTip from '@/composables/sideTip'
import { authServerPreset } from './authServerPreset'

const account = useAccountInfo()
const emit = defineEmits<{ view: [view: ProfileView] }>()
const modal = useModal()

/** CE 的预设认证服务器（`PageLoginAuth.DefaultAuthServer`）。 */
const LITTLE_SKIN = 'https://littleskin.cn/api/yggdrasil'

async function copyUuid(item: McProfile) {
  try {
    await navigator.clipboard.writeText(item.uuid)
    sideTip.show(pclText('Proto.Clipboard.CopySuccess'), 'success')
  } catch {
    sideTip.show(pclText('Proto.Clipboard.CopyFailed'), 'warn')
  }
}

/** 对照 CE 的 `ProfileUi.EditOfflineUuid`：先选类型，自定义时再输入。 */
async function changeUuid(item: McProfile) {
  const selection = await modal.open({
    title: pclText('Launch.Account.Profile.Uuid.SelectType.Title'),
    width: ModalWidthVariant.Slim,
    selectOptions: [
      { value: 'standard', label: pclText('Launch.Account.Profile.Uuid.Standard') },
      { value: 'legacy', label: pclText('Launch.Account.Profile.Uuid.Legacy') },
      { value: 'custom', label: pclText('Common.Option.Customize') },
    ],
    confirmText: pclText('Common.Action.Continue'),
    cancelText: pclText('Common.Action.Cancel'),
  })
  if (!selection.confirmed || !selection.selected) return
  const mode = selection.selected as UuidMode
  let customUuid: string | undefined
  if (mode === 'custom') {
    const input = await modal.open({
      title: pclText('Launch.Account.Profile.Uuid.ChangeTitle', [item.userName]),
      content: pclText('Launch.Account.Profile.Uuid.Hint'),
      width: ModalWidthVariant.Slim,
      showInput: true,
      defaultInputText: item.uuid,
      confirmText: pclText('Common.Action.Continue'),
      cancelText: pclText('Common.Action.Cancel'),
    })
    customUuid = input.confirmed ? (input.input ?? '').replaceAll('-', '') : undefined
    if (!customUuid || !/^[a-fA-F0-9]{32}$/.test(customUuid)) {
      if (customUuid) sideTip.show(pclText('Launch.Account.Profile.Uuid.InvalidChars'), 'warn')
      return
    }
  }
  try {
    await account.setOfflineUuid(item.profileId, mode, customUuid)
    sideTip.show(pclText('Launch.Account.Profile.Saved'), 'success')
  } catch (error) {
    sideTip.show(toAuthFailure(error).message, 'warn')
  }
}

/** 对照 CE 的 `DeleteProfile`：二次确认后删除。 */
async function remove(item: McProfile) {
  const { confirmed } = await dialog({
    title: pclText('Launch.Account.Profile.DeleteConfirm.Title'),
    content: pclText('Launch.Account.Profile.DeleteConfirm.Message'),
    ok: pclText('Common.Action.Continue'),
    cancel: pclText('Common.Action.Cancel'),
  })
  if (!confirmed) return
  try {
    await account.remove(item.profileId)
    sideTip.show(pclText('Launch.Account.Profile.Deleted'), 'success')
  } catch (error) {
    sideTip.show(toAuthFailure(error).message, 'warn')
  }
}

async function choose(item: McProfile) {
  try {
    await account.select(item.profileId)
    emit('view', 'skin')
  } catch (error) {
    sideTip.show(toAuthFailure(error).message, 'warn')
  }
}

/** 第三方登录先选服务器，对照 CE 的 `MyMsgAuthServer` 弹窗。 */
async function chooseAuthServer() {
  const selection = await modal.open({
    title: pclText('Launch.Account.Auth.SelectServer'),
    width: ModalWidthVariant.Slim,
    selectOptions: [
      { value: LITTLE_SKIN, label: pclText('Launch.Account.Auth.Preset.LittleSkin') },
      { value: 'custom', label: pclText('Common.Option.Customize') },
    ],
    confirmText: pclText('Common.Action.Continue'),
    cancelText: pclText('Common.Action.Cancel'),
  })
  if (!selection.confirmed || !selection.selected) return false
  if (selection.selected === 'custom') {
    const input = await modal.open({
      title: pclText('Launch.Account.Auth.SelectServer'),
      content: pclText('Launch.Account.Auth.ServerLabel', ['']),
      width: ModalWidthVariant.Slim,
      showInput: true,
      defaultInputText: 'https://',
      confirmText: pclText('Common.Action.Continue'),
      cancelText: pclText('Common.Action.Cancel'),
    })
    const server = (input.confirmed ? (input.input ?? '') : '').trim()
    if (!/^https?:\/\//i.test(server)) {
      if (server) sideTip.show(pclText('Launch.Account.Auth.InvalidServer'), 'warn')
      return false
    }
    authServerPreset.value = server
    return true
  }
  authServerPreset.value = selection.selected
  return true
}

async function createProfile() {
  const selectOptions = [
    {
      value: 'microsoft',
      label: pclText('Launch.Account.Type.Microsoft'),
      icon: 'lucide/shield-check',
    },
    {
      value: 'auth',
      label: pclText('Launch.Account.Type.ThirdParty'),
      icon: 'lucide/network',
    },
    {
      value: 'offline',
      label: pclText('Launch.Account.Type.Offline'),
      icon: 'lucide/link-2-off',
    },
  ]
  const result = await modal.open({
    title: pclText('Launch.Account.Profile.Create.SelectAuthType.Title'),
    width: ModalWidthVariant.Slim,
    selectOptions,
    confirmText: pclText('Common.Action.Continue'),
    cancelText: pclText('Common.Action.Cancel'),
  })
  if (!result.confirmed || !result.selected) return
  if (result.selected === 'auth' && !(await chooseAuthServer())) return
  emit('view', result.selected as ProfileView)
}
</script>
<template>
  <section class="account-list">
    <PHint v-if="account.profiles.length === 0" severity="info">{{
      pclText('Launch.Account.Profile.CreateAndSelectHint')
    }}</PHint>
    <PListItem
      v-for="item in account.profiles"
      :key="item.profileId"
      class="profile-item"
      :title="item.userName"
      :subtitle="getProfileInfo(item)"
      clickable
      @click="() => choose(item)"
    >
      <template #icon>
        <div class="profile-head">
          <PSkin :skin-url="account.skinFor(item.profileId)" />
        </div>
      </template>
      <template #icon-actions>
        <button
          v-if="item.profileType === 'offline'"
          class="pcl-icon-button"
          type="button"
          :title="pclText('Launch.Account.Profile.ChangeUuid')"
          :aria-label="pclText('Launch.Account.Profile.ChangeUuid')"
          @click.stop="changeUuid(item)"
        >
          <PSvgIcon name="lucide/pencil" :size="15" />
        </button>
        <button
          v-else
          class="pcl-icon-button"
          type="button"
          :title="pclText('Launch.Account.Profile.CopyUuid')"
          :aria-label="pclText('Launch.Account.Profile.CopyUuid')"
          @click.stop="copyUuid(item)"
        >
          <PSvgIcon name="lucide/copy" :size="15" />
        </button>
        <button
          class="pcl-icon-button"
          type="button"
          :title="pclText('Launch.Account.Profile.Delete')"
          :aria-label="pclText('Launch.Account.Profile.Delete')"
          @click.stop="remove(item)"
        >
          <PSvgIcon name="lucide/trash-2" :size="15" />
        </button>
      </template>
    </PListItem>
    <PCard hide-title class="create-card">
      <button
        class="pcl-icon-button create-profile"
        type="button"
        :aria-label="pclText('Launch.Account.Profile.Create')"
        :title="pclText('Launch.Account.Profile.Create')"
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
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
  transform: translate(-50%, -50%) scale(0.5357);
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
