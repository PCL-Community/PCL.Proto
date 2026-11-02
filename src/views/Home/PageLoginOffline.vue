<script setup lang="ts">
import { ref } from 'vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import PButton from '@/components/widget/PButton.vue'
import { useAccountInfo } from '@/stores/account'
import type { ProfileView } from '@/types/profile'
import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
const emit = defineEmits<{ view: [view: ProfileView] }>()
const text = pclText
const username = ref(useAccountInfo().username || '')
const uuidType = ref('standard')
const uuid = ref('')
function create() {
  if (!/^[A-Za-z0-9_]{3,16}$/.test(username.value)) {
    sideTip.show(text('Launch.Account.Profile.EditPlayerId.NotAllowed'), 'warn')
    return
  }
  if (uuidType.value === 'custom' && !/^[a-f0-9]{32}$/i.test(uuid.value)) {
    sideTip.show(text('Launch.Account.Offline.Uuid.Custom.Input.ToolTip'), 'warn')
    return
  }
  sideTip.show(pclText('Proto.NotImplemented.Profile'))
}
</script>
<template>
  <form class="account-form offline-form" @submit.prevent="create">
    <label class="form-row"
      ><span>{{ text('Launch.Account.Offline.PlayerId') }}</span
      ><PTextBox v-model="username" /></label
    ><strong>{{ text('Launch.Account.Offline.UuidStandard') }}</strong>
    <div class="uuid-types">
      <label v-for="kind in ['standard', 'legacy', 'custom']" :key="kind"
        ><input type="radio" v-model="uuidType" :value="kind" />{{
          text(
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
      <PButton compact @click="() => emit('view', 'profiles')">{{
        text('Launch.Account.Offline.Back')
      }}</PButton
      ><PButton compact type="tint" @click="create">{{
        text('Launch.Account.Offline.Create')
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
