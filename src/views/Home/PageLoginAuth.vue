<script setup lang="ts">
import { ref } from 'vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import PButton from '@/components/widget/PButton.vue'
import type { ProfileView } from '@/types/profile'
import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import PPath from '@/components/widget/PPath.vue'
import paths from '@/assets/vector-paths.json'
import { openExternal } from '@/util/externalLink'
const loginShape = paths.a734eac303dcf223.data
defineProps<{ website?: string; registerUrl?: string }>()
const oauthMode = ref(false)
const emit = defineEmits<{ view: [view: ProfileView] }>()
const text = pclText
const server = ref('')
const username = ref('')
const password = ref('')
function login() {
  if (!/^https?:\/\//i.test(server.value)) {
    sideTip.show(text('Launch.Account.Auth.InvalidServer'), 'warn')
    return
  }
  if (!oauthMode.value && (!username.value || !password.value)) {
    sideTip.show(text('Launch.Account.Auth.EmptyFields'), 'warn')
    return
  }
  sideTip.show(pclText('Proto.NotImplemented.Auth'))
}
</script>
<template>
  <form class="account-form auth-form" @submit.prevent="login">
    <label class="server-row">
      <span>{{ text('Launch.Account.Auth.SelectServer') }}</span>
      <PTextBox v-model="server" placeholder="https://" />
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
      <PButton
        v-if="registerUrl"
        class="account-text register-link"
        inline
        @click="() => openExternal(registerUrl)"
        >{{ text('Launch.Account.Auth.Register') }}</PButton
      >
      <div class="account-links auth-links">
        <PButton v-if="website" inline @click="() => openExternal(website)">{{
          text('Launch.Account.Microsoft.Website')
        }}</PButton>
        <PButton inline @click="() => (oauthMode = true)">{{
          text('Launch.Account.Auth.UseOAuth')
        }}</PButton>
      </div>
    </template>
    <div v-else class="oauth-login">
      <PPath :data="loginShape" /><PButton type="tint" compact @click="login">{{
        text('Launch.Account.Auth.OAuthLogin')
      }}</PButton>
      <div class="account-links auth-links">
        <PButton v-if="website" inline @click="() => openExternal(website)">{{
          text('Launch.Account.Microsoft.Website')
        }}</PButton>
        <PButton inline @click="() => (oauthMode = false)">{{
          text('Launch.Account.Auth.UsePassword')
        }}</PButton>
      </div>
    </div>
    <div v-if="!oauthMode" class="auth-actions">
      <PButton compact @click="() => emit('view', 'profiles')">{{
        text('Launch.Account.Auth.Back')
      }}</PButton
      ><PButton compact type="tint" @click="login">{{ text('Launch.Account.Auth.Login') }}</PButton>
    </div>
    <PButton
      v-else
      class="account-back oauth-back"
      inline
      @click="() => emit('view', 'profiles')"
      >{{ text('Launch.Account.Back') }}</PButton
    >
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
