<script setup lang="ts">
import PButton from '@/components/widget/PButton.vue'
import PPath from '@/components/widget/PPath.vue'
import paths from '@/assets/vector-paths.json'
const loginShape = paths['05803c336117c503'].data
import type { ProfileView } from '@/types/profile'
import { computed } from 'vue'
import { pclText } from '@/api/pclLocale'
import { openExternal } from '@/util/externalLink'
import { useAccountInfo } from '@/stores/account'
import { loginProgress } from '@/composables/useLoginSession'

const emit = defineEmits<{ view: [view: ProfileView] }>()
const account = useAccountInfo()
// 登录会话由启动页统一驱动，避免登录途中切页丢掉弹窗与结果提示。
const progress = loginProgress('microsoft')
const loggingIn = computed(() => account.loggingIn)

function login() {
  void account.startLogin({ profileType: 'microsoft' })
}

function back() {
  account.resetLoginState()
  emit('view', 'profiles')
}
</script>
<template>
  <section class="ms-login">
    <PPath :data="loginShape" class="login-avatar" /><PButton
      class="login-button"
      type="tint"
      compact
      :disabled="loggingIn"
      @click="login"
      >{{ progress ?? pclText('Launch.Account.Microsoft.Start') }}</PButton
    >
    <div class="account-links login-links">
      <PButton
        inline
        @click="
          () => openExternal('https://www.minecraft.net/store/minecraft-java-bedrock-edition-pc')
        "
        >{{ pclText('Launch.Account.Microsoft.Purchase') }}</PButton
      >
      <PButton inline @click="() => openExternal('https://www.minecraft.net')">{{
        pclText('Launch.Account.Microsoft.Website')
      }}</PButton>
    </div>
    <PButton v-if="!loggingIn" class="account-back login-back" inline @click="back">{{
      pclText('Launch.Account.Back')
    }}</PButton>
  </section>
</template>
<style scoped>
.ms-login {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  margin-top: 10px;
}
.login-avatar {
  width: 40px;
  height: 40px;
  opacity: 0.7;
  color: var(--ce-color-2);
  margin-left: 4px;
}
.login-button {
  width: 100px;
  margin-top: 25px;
}
.login-links {
  margin-top: 30px;
}
.login-back {
  margin-top: 15px;
}
</style>
