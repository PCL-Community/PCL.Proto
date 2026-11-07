<script setup lang="ts">
import { computed, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import PSkin from '@/components/widget/PSkin.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import PCard from '@/components/widget/PCard.vue'
import { useAccountInfo, toAuthFailure, type McProfile } from '@/stores/account'
import type { ProfileView } from '@/types/profile'
import { pclText } from '@/api/pclLocale'
import { popupNativeMenu, type NativeMenuItem } from '@/util/nativeMenu'
import { authAccountUrl, authClosetUrl, getProfileInfo } from '@/util/profile'
import { openExternal } from '@/util/externalLink'
import { dialog, loginUrls } from '@/util/loginErrors'
import sideTip from '@/composables/sideTip'
import { useModal } from '@/composables/useModal'
import { ModalWidthVariant } from '@/types/modal'
import { showUnavailable } from '@/util/showUnavailable'

const account = useAccountInfo()
const emit = defineEmits<{ view: [view: ProfileView] }>()
const modal = useModal()
const busy = ref(false)

const website = !isTauri()
const profile = computed<McProfile | null>(() =>
  website
    ? {
        profileId: 'website',
        userName: 'AMagicPear',
        uuid: '31bbe5379fea4e68aa4ad7aacca23d13',
        profileType: 'microsoft',
        description: '',
      }
    : account.profile,
)
// CE：离线档案不显示“修改信息”按钮。
const canEdit = computed(() => !!profile.value && profile.value.profileType !== 'offline')

/** 对照 CE 的 `Skin_Click`：正版上传皮肤，第三方跳转皮肤站衣柜。 */
async function changeSkin() {
  const current = profile.value
  if (!current) return
  if (current.profileType === 'microsoft') {
    await changeLocalSkin(current.profileId, false)
    return
  }
  if (current.profileType === 'offline') {
    await changeLocalSkin(current.profileId, true)
    return
  }
  const closet = authClosetUrl(current.server)
  if (closet) {
    await openExternal(closet)
    return
  }
  sideTip.show(pclText('Launch.Account.ProfileSkin.SkinUnsupported'), 'warn')
}

/** 对照 CE 的 `ProfileUi.ChangeSkinMs`：选文件 → 选模型 → 上传。 */
async function changeLocalSkin(profileId: string, offline: boolean) {
  if (busy.value) {
    sideTip.show(pclText('Launch.Skin.Change.Busy'))
    return
  }
  const source = await open({
    title: pclText('Launch.Skin.FileDialog.Title'),
    multiple: false,
    filters: [
      {
        name: pclText('Launch.Skin.FileDialog.Filter'),
        extensions: offline ? ['png'] : ['png', 'jpg', 'webp'],
      },
    ],
  })
  if (!source || typeof source !== 'string') return
  const { choice } = await dialog({
    title: pclText('Launch.Skin.Model.SelectTitle'),
    content: pclText('Launch.Skin.Model.SelectMessage'),
    ok: pclText('Launch.Skin.Model.Steve'),
    cancel: pclText('Launch.Skin.Model.Alex'),
    extra: pclText('Common.Option.IDontKnow'),
  })
  if (choice !== 1 && choice !== 2) {
    sideTip.show(pclText('Launch.Skin.Model.UnknownHint'))
    return
  }
  busy.value = true
  sideTip.show(pclText('Launch.Skin.Change.Starting'))
  try {
    await invoke(offline ? 'change_offline_skin' : 'change_microsoft_skin', {
      profileId,
      source,
      slim: choice === 2,
    })
    sideTip.show(pclText('Launch.Skin.ChangeSuccess'), 'success')
    await account.loadSkin(profileId, true)
  } catch (error) {
    const failure = toAuthFailure(error)
    await dialog({
      title: pclText('Launch.Account.Profile.Error.ChangeSkin'),
      content: pclText('Launch.Skin.Change.FailedWithDetail', [failure.message]),
      ok: pclText('Minecraft.Launch.Login.IKnow'),
    })
  } finally {
    busy.value = false
  }
}

async function resetOfflineSkin() {
  const current = profile.value
  if (!current || busy.value) return
  busy.value = true
  try {
    await invoke('change_offline_skin', { profileId: current.profileId, source: null, slim: false })
    await account.loadSkin(current.profileId, true)
  } catch (error) {
    sideTip.show(toAuthFailure(error).message, 'warn')
  } finally {
    busy.value = false
  }
}

/** 对照 CE 的 `MySkin.Save`：另存当前显示的皮肤。 */
async function saveSkin() {
  const current = profile.value
  if (!current) return
  const url = account.skinFor(current.profileId)
  const destination = await save({
    title: pclText('Launch.Skin.SaveDialog.Title'),
    defaultPath: `${current.userName}.png`,
    filters: [{ name: pclText('Launch.Skin.SaveDialog.Filter'), extensions: ['png'] }],
  })
  if (!destination) return
  try {
    const bytes = await (await fetch(url)).arrayBuffer()
    const image = bytesToDataUrl(bytes)
    await invoke('save_profile_skin', { destination, image })
    sideTip.show(pclText('Launch.Skin.SaveSuccess'), 'success')
  } catch (error) {
    console.warn('Failed to save the skin', error)
    sideTip.show(pclText('Launch.Skin.Save.Error'), 'warn')
  }
}

function bytesToDataUrl(bytes: ArrayBuffer) {
  const chunks: string[] = []
  const view = new Uint8Array(bytes)
  for (let index = 0; index < view.length; index += 0x8000) {
    chunks.push(String.fromCharCode(...view.subarray(index, index + 0x8000)))
  }
  return `data:image/png;base64,${btoa(chunks.join(''))}`
}

/** 对照 CE 的 `MySkin.RefreshCache`：丢掉缓存重新拉取。 */
async function refreshSkin() {
  const current = profile.value
  if (!current) return
  sideTip.show(pclText('Launch.Skin.Refreshing'))
  await account.loadSkin(current.profileId, true)
  sideTip.show(pclText('Launch.Skin.RefreshSuccess'), 'success')
}

/** 对照 CE 的 `BtnSkinCape_Click`：第三方跳转衣柜，离线不支持。 */
async function changeCape() {
  const current = profile.value
  if (!current) return
  if (current.profileType === 'microsoft') {
    // CE 会拉取披风列表并渲染预览；Proto 暂未实现披风预览。
    sideTip.show(pclText('Launch.Account.ProfileSkin.CapeUnsupported'), 'warn')
    return
  }
  const closet = authClosetUrl(current.server)
  if (closet) {
    await openExternal(closet)
    return
  }
  sideTip.show(pclText('Launch.Account.ProfileSkin.CapeUnsupported'), 'warn')
}

/** 对照 CE 的 `BtnEditPassword_Click`。 */
async function editPassword() {
  const current = profile.value
  if (!current) return
  if (current.profileType === 'microsoft') {
    await openExternal(loginUrls.password)
    return
  }
  const accountUrl = authAccountUrl(current.server)
  if (accountUrl) {
    await openExternal(accountUrl)
    return
  }
  sideTip.show(pclText('Launch.Account.ProfileSkin.PasswordUnsupported'), 'warn')
}

/** 对照 CE 的 `ProfileUi.EditProfileId`：正版改名，第三方跳转皮肤站。 */
async function editUsername() {
  const current = profile.value
  if (!current) return
  if (current.profileType !== 'microsoft') {
    const accountUrl = authAccountUrl(current.server)
    if (accountUrl) await openExternal(accountUrl)
    return
  }
  const input = await modal.open({
    title: pclText('Launch.Account.Profile.EditPlayerId.Title'),
    content: pclText('Launch.Account.Profile.EditPlayerId.MicrosoftWarning'),
    width: ModalWidthVariant.Slim,
    showInput: true,
    defaultInputText: current.userName,
    confirmText: pclText('Common.Action.Confirm'),
    cancelText: pclText('Common.Action.Cancel'),
  })
  const name = (input.confirmed ? (input.input ?? '') : '').trim()
  if (!name) return
  if (!/^[A-Za-z0-9_]{3,16}$/.test(name)) {
    sideTip.show(pclText('Launch.Account.Profile.EditPlayerId.NotAllowed'), 'warn')
    return
  }
  const { confirmed } = await dialog({
    title: pclText('Launch.Account.Profile.EditPlayerId.Confirm.Title'),
    content: pclText('Launch.Account.Profile.EditPlayerId.Confirm.Message'),
    ok: pclText('Common.Action.Continue'),
    cancel: pclText('Common.Action.Cancel'),
  })
  if (!confirmed) return
  try {
    await account.renameMicrosoft(current.profileId, name)
    sideTip.show(pclText('Launch.Account.Profile.EditPlayerId.Success', [name]), 'success')
  } catch (error) {
    const failure = toAuthFailure(error)
    const message =
      failure.code === 'duplicate_name'
        ? pclText('Launch.Account.Profile.EditPlayerId.Duplicate')
        : failure.code === 'not_allowed'
          ? pclText('Launch.Account.Profile.EditPlayerId.NotAllowed')
          : failure.code === 'name_change_cooldown'
            ? pclText('Launch.Account.Profile.EditPlayerId.Cooldown')
            : failure.message
    await dialog({
      title: pclText('Launch.Account.Profile.EditPlayerId.Failed.Title'),
      content: message,
      ok: pclText('Common.Action.Confirm'),
    })
  }
}

/** 对照 CE 的 `ChangeProfile`：清空当前档案并回到档案列表。 */
async function switchProfile() {
  if (website) {
    void showUnavailable()
    return
  }
  await account.select(null)
  emit('view', 'profiles')
}

const skinActions = ['ChangeSkin', 'SaveSkin', 'RefreshSkin', 'ChangeCape'] as const
const editActions = ['EditPassword', 'EditUsername'] as const

function runAction(action: string) {
  switch (action) {
    case 'ChangeSkin':
      void changeSkin()
      return
    case 'SaveSkin':
      void saveSkin()
      return
    case 'RefreshSkin':
      void refreshSkin()
      return
    case 'ResetOfflineSkin':
      void resetOfflineSkin()
      return
    case 'ChangeCape':
      void changeCape()
      return
    case 'EditPassword':
      void editPassword()
      return
    case 'EditUsername':
      void editUsername()
      return
  }
}

async function openMenu(event: MouseEvent, actions: readonly string[]) {
  if (website) {
    void showUnavailable()
    return
  }
  const available =
    actions === skinActions && profile.value?.profileType === 'offline'
      ? [...actions, 'ResetOfflineSkin']
      : actions
  const items: NativeMenuItem[] = available.map((action) => ({
    text:
      action === 'ResetOfflineSkin'
        ? pclText('Proto.Account.ResetOfflineSkin')
        : pclText(`Launch.Account.ProfileSkin.${action}`),
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
  <div v-if="profile" class="profile-skin">
    <PSkin
      class="profile-avatar"
      :skin-url="
        website ? 'https://minotar.net/skin/AMagicPear' : account.skinFor(profile?.profileId)
      "
    /><strong>{{ profile?.userName || 'Player' }}</strong
    ><span :title="getProfileInfo(profile)">{{ getProfileInfo(profile) }}</span>
    <div class="profile-actions-reveal">
      <PCard hide-title class="profile-actions-card"
        ><div class="profile-actions">
          <button
            type="button"
            class="pcl-icon-button profile-action"
            :aria-label="pclText('Launch.Account.ProfileSkin.SkinsAndCapes')"
            :title="pclText('Launch.Account.ProfileSkin.SkinsAndCapes')"
            @click="openMenu($event, skinActions)"
          >
            <PSvgIcon name="lucide/shirt" :size="14" />
          </button>
          <button
            v-if="canEdit"
            type="button"
            class="pcl-icon-button profile-action"
            :aria-label="pclText('Launch.Account.ProfileSkin.EditProfile')"
            :title="pclText('Launch.Account.ProfileSkin.EditProfile')"
            @click="openMenu($event, editActions)"
          >
            <PSvgIcon name="lucide/pencil" :size="14" />
          </button>
          <button
            type="button"
            class="pcl-icon-button profile-action"
            @click="switchProfile"
            :aria-label="pclText('Launch.Account.ProfileSkin.SwitchProfile')"
            :title="pclText('Launch.Account.ProfileSkin.SwitchProfile')"
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
  max-width: 100%;
  overflow: hidden;
  color: var(--color-text-grey);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
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
