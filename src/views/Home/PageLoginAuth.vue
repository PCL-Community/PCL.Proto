<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import PButton from '@/components/widget/PButton.vue'
import type { ProfileView } from '@/types/profile'
import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import PPath from '@/components/widget/PPath.vue'
import paths from '@/assets/vector-paths.json'
import { openExternal } from '@/util/externalLink'
import { useAccountInfo } from '@/stores/account'
import { loginProgress } from '@/composables/useLoginSession'
import { authServerPreset } from './authServerPreset'

const loginShape = paths.a734eac303dcf223.data
const props = defineProps<{ registerUrl?: string }>()
const emit = defineEmits<{ view: [view: ProfileView] }>()
const text = pclText
const account = useAccountInfo()
// 登录会话由启动页统一驱动，避免登录途中切页丢掉弹窗与结果提示。
const progress = loginProgress('auth')
const loggingIn = computed(() => account.loggingIn)

const server = ref('')
const username = ref('')
const password = ref('')
const oauthMode = ref(false)
/** 探测结果，对照 CE 的 `_oauthSupported` / `ApiLocation.TryRequestAsync`。 */
const probe = ref<{
  apiLocation: string
  oauthSupported: boolean
  discoveryAddress?: string | null
  clientId?: string | null
  website?: string | null
} | null>(null)
let probeTimer: ReturnType<typeof setTimeout> | null = null
let probedAddress = ''

/** 名称为空时是注册模式，否则显示找回密码，对照 CE 的 `ComboName_TextChanged`。 */
const registerMode = ref(true)
watch(username, (value) => {
  registerMode.value = value.trim().length === 0
})

async function probeServer(address: string) {
  if (!/^https?:\/\//i.test(address) || address === probedAddress) return
  try {
    const info = await account.probeServer(address)
    // 用户在探测期间又改了地址时丢弃过期结果（CE 的 `_authServerUrl` 比较）。
    if (server.value !== address) return
    probedAddress = address
    probe.value = info
    oauthMode.value = info.oauthSupported
  } catch (error) {
    console.warn('Failed to probe the authentication server', error)
    if (server.value === address) {
      probedAddress = address
      probe.value = null
      oauthMode.value = false
    }
  }
}

// 地址停顿后再探测，避免每敲一个字符就发请求。
watch(server, (value) => {
  if (probeTimer) clearTimeout(probeTimer)
  probeTimer = setTimeout(() => void probeServer(value.trim()), 500)
})

onMounted(() => {
  if (authServerPreset.value) {
    server.value = authServerPreset.value
    authServerPreset.value = null
    void probeServer(server.value)
  }
})

function login() {
  if (loggingIn.value) return
  const address = server.value.trim()
  if (!address) {
    sideTip.show(text('Launch.Account.Auth.EmptyFields'), 'warn')
    return
  }
  if (!/^https?:\/\//i.test(address)) {
    sideTip.show(text('Launch.Account.Auth.InvalidServer'), 'warn')
    return
  }
  // 探测成功时用解析出的 API 地址，失败时退回用户输入（CE 的 BaseUrl）。
  const apiLocation = probe.value?.apiLocation || address
  if (oauthMode.value) {
    const discovery = probe.value?.discoveryAddress
    if (!discovery) {
      sideTip.show(text('Launch.Account.Auth.LoginFailed'), 'warn')
      return
    }
    void account.startLogin({
      profileType: 'yggdrasilConnect',
      server: apiLocation,
      discoveryAddress: discovery,
      clientId: probe.value?.clientId ?? null,
    })
    return
  }
  if (!username.value.trim() || !password.value) {
    sideTip.show(text('Launch.Account.Auth.EmptyFields'), 'warn')
    return
  }
  void account.startLogin({
    profileType: 'authlib',
    server: apiLocation,
    username: username.value.trim(),
    password: password.value,
  })
}

function back() {
  account.resetLoginState()
  server.value = ''
  username.value = ''
  password.value = ''
  probe.value = null
  probedAddress = ''
  oauthMode.value = false
  emit('view', 'profiles')
}

function openRegister() {
  if (!props.registerUrl) return
  void openExternal(
    registerMode.value
      ? props.registerUrl
      : props.registerUrl.replace('/auth/register', '/auth/forgot'),
  )
}

function openWebsite() {
  void openExternal(probe.value?.website || server.value.trim())
}
</script>
<template>
  <form class="account-form auth-form" @submit.prevent="login">
    <label class="server-row">
      <span>{{ text('Launch.Account.Auth.SelectServer') }}</span>
      <PTextBox v-model="server" placeholder="https://" :disabled="loggingIn" />
    </label>
    <template v-if="!oauthMode">
      <label class="auth-row"
        ><span>{{ text('Launch.Account.Auth.Email') }}</span
        ><PTextBox v-model="username"
      /></label>
      <label class="auth-row"
        ><span>{{ text('Launch.Account.Auth.Password') }}</span
        ><PTextBox type="password" v-model="password" autocomplete="off"
      /></label>
      <PButton v-if="registerUrl" class="account-text register-link" inline @click="openRegister">{{
        text(registerMode ? 'Launch.Account.Auth.Register' : 'Launch.Account.Auth.ForgotPassword')
      }}</PButton>
      <div class="account-links auth-links">
        <PButton inline @click="openWebsite">{{
          text('Launch.Account.Microsoft.Website')
        }}</PButton>
        <PButton v-if="probe?.oauthSupported" inline @click="() => (oauthMode = true)">{{
          text('Launch.Account.Auth.UseOAuth')
        }}</PButton>
      </div>
    </template>
    <div v-else class="oauth-login">
      <PPath :data="loginShape" /><PButton
        type="tint"
        compact
        :disabled="loggingIn"
        @click="login"
        >{{ progress ?? text('Launch.Account.Auth.OAuthLogin') }}</PButton
      >
      <div class="account-links auth-links">
        <PButton inline @click="openWebsite">{{
          text('Launch.Account.Microsoft.Website')
        }}</PButton>
        <PButton inline @click="() => (oauthMode = false)">{{
          text('Launch.Account.Auth.UsePassword')
        }}</PButton>
      </div>
    </div>
    <div v-if="!oauthMode" class="auth-actions">
      <PButton compact :disabled="loggingIn" @click="back">{{
        text('Launch.Account.Auth.Back')
      }}</PButton
      ><PButton compact type="tint" :disabled="loggingIn" @click="login">{{
        progress ?? text('Launch.Account.Auth.Login')
      }}</PButton>
    </div>
    <PButton v-else-if="!loggingIn" class="account-back oauth-back" inline @click="back">{{
      text('Launch.Account.Back')
    }}</PButton>
  </form>
</template>
<style scoped>
.auth-form {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}
.server-row {
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.auth-row {
  display: grid;
  grid-template-columns: 50px minmax(0, 1fr);
  align-items: center;
}
.auth-form :deep(input) {
  height: 28px;
  padding: 0 8px;
}
.register-link {
  align-self: flex-end;
}
.auth-actions,
.auth-links {
  display: flex;
  justify-content: center;
}
.auth-actions {
  gap: 10px;
  margin-top: 10px;
}
.auth-actions > button {
  width: 50px;
  padding: 0;
}
.auth-links {
  margin-top: 10px;
}
.oauth-login {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-top: 10px;
}
.oauth-login > svg {
  width: 40px;
  height: 40px;
  opacity: 0.7;
  color: var(--ce-color-2);
}
.oauth-login > button {
  width: 120px;
  margin-top: 25px;
}
.oauth-login .auth-links {
  margin-top: 30px;
}
.oauth-back {
  align-self: center;
  margin-top: 20px;
}
</style>
