<script setup lang="ts">
import PButton from '@/components/widget/PButton.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PageLoginProfileSkin from './PageLoginProfileSkin.vue'
import PageLoginProfile from './PageLoginProfile.vue'
import PageLoginOffline from './PageLoginOffline.vue'
import PageLoginMs from './PageLoginMs.vue'
import PageLoginAuth from './PageLoginAuth.vue'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { useAccountInfo } from '@/stores/account'
import { useLoginSession } from '@/composables/useLoginSession'
import { getProfileTypeLabel } from '@/util/profile'
import { useRouter } from 'vue-router'
import type { ProfileView } from '@/types/profile'
import { Channel, invoke, isTauri } from '@tauri-apps/api/core'
import { prepareGameJava, downloadGameJava } from '@/api/javaReq'
import { useModal } from '@/composables/useModal'
import { computed, ref, watch } from 'vue'
import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import { ModalWidthVariant } from '@/types/modal'
import { useLaunchState, type LaunchFailure, type LaunchProgress } from '@/stores/launchState'
import { storeToRefs } from 'pinia'

defineProps<{ loadingInstances?: boolean }>()

const router = useRouter()
const instance = useSelectedInstance()
const account = useAccountInfo()
const website = !isTauri()
const profileView = ref<ProfileView>(account.profile ? 'skin' : 'profiles')
const accountViews = {
  skin: PageLoginProfileSkin,
  profiles: PageLoginProfile,
  offline: PageLoginOffline,
  microsoft: PageLoginMs,
  auth: PageLoginAuth,
}
const accountView = computed(() =>
  website ? PageLoginProfileSkin : accountViews[profileView.value],
)
// CE 的 RefreshPage：选中档案时显示皮肤页，没有档案时显示档案列表。
watch(
  () => account.currentId,
  (current) => {
    profileView.value = current ? 'skin' : 'profiles'
  },
)
// 登录会话挂在启动页：设备码弹窗、角色选择与失败提示都在这里，登录途中切页也不会丢。
useLoginSession(
  () => (profileView.value === 'auth' ? 'auth' : 'microsoft'),
  () => {
    profileView.value = 'skin'
  },
)
const modal = useModal()
const launchState = useLaunchState()
const { preparing, launching, downloadSpeed, leavingLaunch } = storeToRefs(launchState)

/** 启动页显示的验证方式，对照 CE 的 `LabLaunchingMethod`。 */
function getLaunchMethod() {
  return account.profile
    ? getProfileTypeLabel(account.profile)
    : pclText('Launch.Account.Type.Offline')
}
function startLaunching() {
  return launchState.start(instance.instance_info?.name || '', getLaunchMethod())
}
function returnToHome(attempt: number) {
  launchState.returnToHome(attempt)
}
async function abandonLaunch(attempt: number) {
  if (!launchState.isCurrent(attempt)) return
  try {
    await invoke('cancel_launch')
  } catch {
    // The Rust side may already have cleared a failed pre-launch request.
  }
}
async function cancelLaunch() {
  const attempt = launchState.requestCancel()
  if (attempt === null) return
  try {
    await invoke('cancel_launch')
    sideTip.show(pclText('Minecraft.Launch.Cancelled'))
  } catch (error) {
    sideTip.show(pclText('Proto.Error.Launch', [String(error)]), 'warn')
  }
  returnToHome(attempt)
}
function formatLaunchFailure(failure: LaunchFailure) {
  const [reason, suggestion] = failure.solution
    ? [
        pclText('Crash.Reason.FabricSolution.WithDetail', [failure.solution]),
        pclText('Crash.Suggestion.FollowLoaderInstructions'),
      ]
    : failure.details
      ? [
          pclText('Crash.Reason.IncompatibleMods.WithDetail', [failure.details]),
          pclText('Crash.Suggestion.FollowLoaderInstructions'),
        ]
      : [pclText('Crash.Reason.Unknown.Auto'), pclText('Crash.Suggestion.ExportReport')]
  return pclText('Crash.Presentation.WithSuggestion', [reason, suggestion])
}
function showLaunchFailure(attempt: number, failure: LaunchFailure) {
  if (launchState.isCancelled(attempt) || !launchState.isCurrent(attempt)) return
  returnToHome(attempt)
  void modal.open({
    title: pclText('Crash.Dialog.Title.Auto'),
    content: formatLaunchFailure(failure),
    width: ModalWidthVariant.Fat,
    buttons: [{ content: pclText('Common.Action.Confirm'), operation: () => modal.close() }],
  })
}
async function launch(serverAddress?: string) {
  if (preparing.value || account.loggingIn) return
  // CE：没有选中档案时不允许启动。
  if (!account.profile) {
    profileView.value = 'profiles'
    sideTip.show(pclText('Launch.Account.Profile.SelectHint'), 'warn')
    return
  }
  if (!instance.instance_info) {
    await router.push('/download/game')
    return
  }
  const instanceDirectory = instance.instance_info.directory
  let attempt: number | undefined
  try {
    await invoke('begin_launch')
    attempt = startLaunching()
    launchState.setPreparing(attempt, true)
    launchState.update(attempt, {
      stage: 'Minecraft.Launch.Stage.GetJava',
      progress: 0.05,
      downloading: false,
    })
    let info = await prepareGameJava(instanceDirectory)
    if (launchState.isCancelled(attempt)) return
    if (!info.runtime) {
      if (!info.download_major) {
        await modal.open({
          title: pclText('Minecraft.Launch.Java.NotFound.Title'),
          content: pclText('Proto.Java.ManualRequired', [
            info.requirements.min[0],
            info.requirements.max[0],
          ]),
          buttons: [{ content: pclText('Common.Action.Close'), operation: () => modal.close() }],
        })
        await abandonLaunch(attempt)
        returnToHome(attempt)
        return
      }
      const answer = await modal.open({
        title: pclText('Minecraft.Launch.Java.AutoDownload.Title'),
        content: pclText('Minecraft.Launch.Java.AutoDownload.Message', [
          `Java ${info.download_major}`,
        ]),
        buttons: [
          {
            content: pclText('Minecraft.Launch.Java.AutoDownload.Action'),
            operation: () => modal.close(true),
          },
          { content: pclText('Common.Action.Cancel'), operation: () => modal.close(false) },
        ],
      })
      if (
        launchState.isCancelled(attempt) ||
        !answer.confirmed ||
        instance.instance_info?.directory !== instanceDirectory
      ) {
        await abandonLaunch(attempt)
        returnToHome(attempt)
        return
      }
      await downloadGameJava(instanceDirectory, (progress) => {
        launchState.update(attempt!, {
          stage: 'Minecraft.Launch.Stage.GetJava',
          progress: progress.total ? 0.05 + (progress.completed / progress.total) * 0.15 : 0.05,
          downloading: true,
        })
      })
      if (launchState.isCancelled(attempt)) return
      info = await prepareGameJava(instanceDirectory)
      if (launchState.isCancelled(attempt)) return
      if (!info.runtime) throw new Error(pclText('Proto.Java.ReadFailed'))
    }
    if (launchState.isCancelled(attempt)) return
    for (const warning of info.warnings)
      sideTip.show(pclText(warning.key, [warning.version, warning.min, warning.max]), 'warn')
    const progress = new Channel<LaunchProgress>()
    progress.onmessage = (message) => {
      if (!launchState.update(attempt!, message)) return
      if (message.failure) {
        showLaunchFailure(attempt!, message.failure)
        return
      }
      if (message.progress === 1) returnToHome(attempt!)
    }
    await invoke('launch_game', {
      instanceDirectory,
      progress,
      serverAddress: serverAddress ?? null,
    })
  } catch (err) {
    if (attempt !== undefined && launchState.isCurrent(attempt)) {
      await abandonLaunch(attempt)
      if (!launchState.isCancelled(attempt))
        sideTip.show(pclText('Proto.Error.Launch', [String(err)]), 'warn')
      returnToHome(attempt)
    } else if (attempt === undefined)
      sideTip.show(pclText('Proto.Error.Launch', [String(err)]), 'warn')
  } finally {
    if (attempt !== undefined) launchState.setPreparing(attempt, false)
  }
}
watch(
  () => launchState.pendingServer,
  (pending) => {
    if (!pending) return
    launchState.pendingServer = null
    if (pending.directory === instance.instance_info?.directory && !preparing.value) {
      void launch(pending.address || undefined)
    } else {
      sideTip.show(pclText('Instance.Server.Card.LaunchFailed'), 'warn')
    }
  },
  { immediate: true },
)
function settings() {
  if (instance.instance_info) void router.push('/instance_setting')
  else sideTip.show(pclText('Proto.Instance.SelectFirstHint'))
}
</script>

<template>
  <aside
    class="launch-left"
    :class="{ 'is-launching': launching && !leavingLaunch, 'is-returning': leavingLaunch }"
  >
    <div class="launch-input">
      <div class="login-panel">
        <Transition name="login-page" mode="out-in">
          <component :is="accountView" @view="profileView = $event" />
        </Transition>
      </div>
      <div class="launch-actions">
        <PButton
          class="launch"
          type="tint"
          @click="() => launch()"
          :disabled="loadingInstances || preparing || !account.profile || account.loggingIn"
          ><span>{{
            pclText(
              loadingInstances
                ? 'Launch.Home.Button.Loading'
                : instance.instance_info
                  ? 'Launch.Home.Button.Launch'
                  : 'Launch.Home.Button.Download',
            )
          }}</span
          ><small>{{
            loadingInstances
              ? pclText('Launch.Home.VersionList.Loading')
              : instance.instance_info?.name || pclText('Select.Instance.Empty.Title')
          }}</small></PButton
        ><PButton @click="() => router.push('/instance_select')">{{
          pclText('Launch.Home.SelectInstance')
        }}</PButton
        ><PButton @click="settings">{{ pclText('Launch.Home.InstanceSettings') }}</PButton>
      </div>
    </div>
    <section v-if="launching" class="launch-status" aria-live="polite">
      <PLoading class="launching-loader" :card="false" />
      <h2>
        {{
          pclText(
            launching.progress === 1
              ? 'Launch.Status.Title.Launched'
              : 'Launch.Status.Title.Launching',
          )
        }}
      </h2>
      <p class="launch-name">{{ launching.name }}</p>
      <div class="launch-progress" :aria-label="pclText('Launch.Status.LaunchProgress')">
        <span :style="{ width: `${launching.progress * 100}%` }" />
      </div>
      <dl>
        <dt>{{ pclText('Launch.Status.CurrentStep') }}</dt>
        <dd>{{ pclText(launching.stage) }}</dd>
        <dt>{{ pclText('Launch.Status.LoginMethod') }}</dt>
        <dd>{{ launching.method }}</dd>
        <dt v-if="launching.progress < 1">{{ pclText('Launch.Status.LaunchProgress') }}</dt>
        <dd v-if="launching.progress < 1">{{ `${(launching.progress * 100).toFixed(2)}%` }}</dd>
        <dt v-if="launching.downloading">{{ pclText('Launch.Status.DownloadSpeed') }}</dt>
        <dd v-if="launching.downloading">{{ downloadSpeed || '0 B/s' }}</dd>
      </dl>
      <div v-if="launching.progress === 1" class="launch-trivia">
        <span>{{ pclText('Launch.Status.Trivia') }}</span>
        <p>{{ pclText('Launch.Status.Trivia.Placeholder') }}</p>
      </div>
      <PButton v-if="launching.progress < 1" class="launch-cancel" @click="cancelLaunch">{{
        pclText('Common.Action.Cancel')
      }}</PButton>
    </section>
  </aside>
</template>

<style scoped>
.launch-left {
  position: relative;
  width: 300px;
  flex: 0 0 300px;
  padding: 0 20px 20px;
  overflow: hidden;
  animation:
    launch-left-enter 400ms cubic-bezier(0.34, 1.56, 0.64, 1),
    launch-left-fade-in 100ms ease-out;
}
.launch-input {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  opacity: 1;
  transform: scale(1);
  transition:
    opacity 110ms ease-in 50ms,
    transform 160ms ease-in;
}
.is-launching .launch-input {
  pointer-events: none;
  opacity: 0;
  transform: scale(1.2);
}
.is-returning .launch-input {
  transition:
    opacity 250ms ease-out 50ms,
    transform 300ms cubic-bezier(0.34, 1.56, 0.64, 1) 50ms;
}
.login-panel {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 0;
  overflow-y: auto;
  padding: 15px 0;
}
.login-panel :deep(.account-form),
.login-panel :deep(.account-form input:not([type='radio'])) {
  font-size: 13px;
}
.login-panel :deep(.account-links) {
  display: flex;
  justify-content: center;
  gap: 30px;
  opacity: 0.3;
}
.login-panel :deep(.account-links .mybutton.inline),
.login-panel :deep(.mybutton.inline.account-text),
.login-panel :deep(.mybutton.inline.account-back) {
  font-size: 13px;
}
.login-panel :deep(.mybutton.inline.account-back:not(:hover):not(:active)) {
  color: var(--color-text-black);
}
/* CE PageLaunchLeft.PageChange: 100 ms fade-out, swap, 20 ms pause, 100 ms fade-in. */
.login-page-leave-active {
  animation: login-page-leave 100ms cubic-bezier(0.33, 1, 0.68, 1) forwards;
}
.login-page-enter-active {
  animation: login-page-enter 100ms cubic-bezier(0.32, 0, 0.67, 0) 20ms both;
}
.login-page-enter-from,
.login-page-leave-to {
  opacity: 0;
}
@keyframes login-page-leave {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}
@keyframes login-page-enter {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
@media (prefers-reduced-motion: reduce) {
  .login-page-leave-active,
  .login-page-enter-active {
    animation: none;
  }
  .login-page-enter-from {
    opacity: 1;
  }
}
.launch-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}
.launch-actions > button {
  min-height: 35px;
}
.launch {
  grid-column: 1 / -1;
  height: 54px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}
.launch small {
  font-size: 11px;
  color: var(--color-text-grey);
}
.launch-status {
  position: absolute;
  inset: 0 20px 20px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 0;
  opacity: 0;
  transform: scale(0.8);
  /* CE PageLaunchLeft：缩放 500 ms 回弹、不透明度 150 ms（均延迟 100 ms）。 */
  animation:
    launch-status-enter 500ms 100ms cubic-bezier(0.34, 1.56, 0.64, 1) forwards,
    launch-status-fade-in 150ms 100ms linear forwards;
  text-align: center;
}
.is-returning .launch-status {
  pointer-events: none;
  /* CE PageChangeToLogin：缩放 150 ms 缓出、不透明度 150 ms 线性。 */
  animation:
    launch-status-leave 150ms cubic-bezier(0.33, 1, 0.68, 1) forwards,
    launch-status-fade-out 150ms linear forwards;
}
.launch-cancel {
  position: absolute;
  right: 0;
  bottom: 0;
  left: 0;
  height: 35px;
}
.launching-loader :deep(.loading-container) {
  min-height: 0;
  padding: 4px 0 2px;
}
.launching-loader :deep(.loading-text) {
  display: none;
}
.launching-loader :deep(.load-svg) {
  width: 72px;
  height: 72px;
}
.launch-status h2 {
  margin: 10px 15px 0;
  color: var(--color-tint);
  font-size: 20px;
  font-weight: normal;
  transform: skewX(-3deg);
}
.launch-name {
  margin: 5px 40px 0;
  overflow: hidden;
  color: var(--color-tint);
  font-size: 13.5px;
  text-overflow: ellipsis;
  white-space: nowrap;
  transform: skewX(-3deg);
}
.launch-progress {
  height: 4px;
  margin: 12px 10px 27px;
  overflow: hidden;
  background: color-mix(in srgb, var(--color-text-grey) 40%, transparent);
}
.launch-progress > span {
  display: block;
  height: 100%;
  background: linear-gradient(90deg, var(--color-tint-light), var(--color-tint));
  /* CE PageLaunchLeft：进度条 260 ms 线性。 */
  transition: width 260ms linear;
}
.launch-status dl {
  display: grid;
  grid-template-columns: auto auto;
  justify-content: center;
  gap: 5px 15px;
  margin: 0;
  font-size: 12.5px;
  text-align: left;
}
.launch-status dt {
  color: var(--color-text-grey);
  text-align: right;
  opacity: 0.8;
}
.launch-status dd {
  margin: 0;
}
.launch-trivia {
  width: 260px;
  box-sizing: border-box;
  margin: 16px auto 2px;
  padding: 0 11px 10px;
  border: 1px solid color-mix(in srgb, var(--color-text-grey) 50%, transparent);
  border-radius: 3px;
  font-size: 12.5px;
}
.launch-trivia > span {
  display: inline-block;
  padding: 0 8px;
  color: var(--color-text-grey);
  background: var(--color-background);
  transform: translateY(-9px);
}
.launch-trivia p {
  margin: -3px 0 0;
  line-height: 1.45;
}
@keyframes launch-status-enter {
  to {
    transform: scale(1);
  }
}
@keyframes launch-status-fade-in {
  to {
    opacity: 1;
  }
}
@keyframes launch-left-enter {
  from {
    transform: scale(0.96);
  }
  to {
    transform: scale(1);
  }
}
@keyframes launch-left-fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
@keyframes launch-status-leave {
  to {
    transform: scale(0.8);
  }
}
@keyframes launch-status-fade-out {
  to {
    opacity: 0;
  }
}
</style>
