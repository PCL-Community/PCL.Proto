<script setup lang="ts">
import { ref } from 'vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import PButton from '@/components/widget/PButton.vue'
import { useAccountInfo, toAuthFailure, type UuidMode } from '@/stores/account'
import type { ProfileView } from '@/types/profile'
import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import { dialog } from '@/util/loginErrors'

const emit = defineEmits<{ view: [view: ProfileView] }>()
const account = useAccountInfo()
const username = ref(account.username || '')
const uuidType = ref<UuidMode>('standard')
const uuid = ref('')
const creating = ref(false)
const uuidModes: UuidMode[] = ['standard', 'legacy', 'custom']

/** 对照 CE 的 `PageLoginOffline.BtnLogin_Click`：ID 不规范时允许用户坚持创建。 */
async function create() {
  if (creating.value) return
  const name = username.value.trim()
  if (!/^[A-Za-z0-9_]{3,16}$/.test(name)) {
    const { confirmed } = await dialog({
      title: pclText('Launch.Account.Offline.InvalidPlayerId.Title'),
      content: pclText('Launch.Account.Offline.InvalidPlayerId.Message'),
      ok: pclText('Common.Action.Continue'),
      cancel: pclText('Common.Action.Cancel'),
    })
    if (!confirmed) return
  }
  if (uuidType.value === 'custom' && !/^[a-fA-F0-9]{32}$/.test(uuid.value.replaceAll('-', ''))) {
    sideTip.show(pclText('Launch.Account.Offline.Uuid.Custom.Input.ToolTip'), 'warn')
    return
  }
  creating.value = true
  try {
    await account.createOffline(name, uuidType.value, uuid.value.replaceAll('-', '') || undefined)
    sideTip.show(pclText('Launch.Account.Profile.Created'), 'success')
    emit('view', 'skin')
  } catch (error) {
    const failure = toAuthFailure(error)
    sideTip.show(failure.message, 'warn')
  } finally {
    creating.value = false
  }
}

function back() {
  emit('view', 'profiles')
}
</script>
<template>
  <form class="account-form offline-form" @submit.prevent="create">
    <label class="form-row"
      ><span>{{ pclText('Launch.Account.Offline.PlayerId') }}</span
      ><PTextBox v-model="username" /></label
    ><strong>{{ pclText('Launch.Account.Offline.UuidStandard') }}</strong>
    <div class="uuid-types">
      <label v-for="kind in uuidModes" :key="kind"
        ><input type="radio" v-model="uuidType" :value="kind" />{{
          pclText(
            kind === 'custom'
              ? 'Common.Option.Customize'
              : `Launch.Account.Offline.Uuid.${kind === 'standard' ? 'Standard' : 'Legacy'}`,
          )
        }}</label
      >
    </div>
    <label v-if="uuidType === 'custom'" class="form-row"
      ><span>UUID</span><PTextBox v-model="uuid"
    /></label>
    <div class="form-actions">
      <PButton compact :disabled="creating" @click="back">{{
        pclText('Launch.Account.Offline.Back')
      }}</PButton
      ><PButton compact type="tint" :disabled="creating" @click="create">{{
        pclText('Launch.Account.Offline.Create')
      }}</PButton>
    </div>
  </form>
</template>
<style scoped>
.offline-form {
  display: flex;
  flex-direction: column;
  width: 100%;
}
.form-row {
  display: grid;
  grid-template-columns: 50px minmax(0, 1fr);
  align-items: center;
}
.form-row > span {
  text-align: center;
}
.form-row :deep(input) {
  height: 28px;
  padding: 0 8px;
}
.offline-form > strong {
  text-align: center;
  font-weight: bold;
  margin-top: 10px;
}
.uuid-types,
.form-actions {
  display: flex;
  justify-content: center;
  gap: 10px;
}
.uuid-types {
  margin: 4px 0 20px;
}
.uuid-types label {
  display: flex;
  align-items: center;
  gap: 5px;
}
.form-actions {
  margin-top: 20px;
}
.form-actions > button {
  width: 50px;
  padding: 0;
}
</style>
