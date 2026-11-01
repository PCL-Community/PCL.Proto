<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import { computed, ref, useId } from 'vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PIconTextButton from '@/components/widget/PIconTextButton.vue'
import PLoading, { type LoadingState } from '@/components/widget/PLoading.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import sideTip from '@/composables/sideTip'
import { useModal } from '@/composables/useModal'

const id = useId()
const count = ref(0)
const loading = ref<LoadingState>('loading')
const playerName = ref('Player')
const editable = ref(true)
const edition = ref('release')
const editions = computed(() => [
  { key: 'release', text: pclText('Download.Comp.Detail.FileList.ReleaseType.Release') },
  { key: 'snapshot', text: pclText('Download.Comp.Detail.VersionGroup.Snapshot') },
  { key: 'old', text: pclText('Download.Comp.Detail.VersionGroup.Old') },
])
const modal = useModal()

async function showDialog(withInput: boolean) {
  const result = await modal.open({
    title: withInput ? pclText('Proto.Demo.InputDialog') : pclText('Proto.Demo.ConfirmDialog'),
    content: withInput ? pclText('Proto.Demo.InputPrompt') : pclText('Proto.Demo.ConfirmPrompt'),
    plainText: true,
    showInput: withInput,
    defaultInputText: playerName.value,
    buttons: [
      {
        type: 'tint',
        content: pclText('Proto.Action.Confirm'),
        operation: () => modal.close(true),
      },
      { content: pclText('Common.Action.Cancel'), operation: () => modal.close(false) },
    ],
  })
  sideTip.show(
    result.confirmed
      ? withInput
        ? pclText('Proto.Demo.Hello', [result.input || 'Player'])
        : pclText('Proto.Demo.Confirmed')
      : pclText('Proto.Demo.Cancelled'),
    result.confirmed ? 'success' : 'default',
  )
}
</script>

<template>
  <div class="component-showcase">
    <PButton type="tint" @click="() => sideTip.show(pclText('Proto.Demo.Clicked', [++count]))">{{
      pclText('Proto.Demo.ClickHint')
    }}</PButton>
    <PCard :title="pclText('Proto.Demo.Buttons')" default-fold-status="unfold">
      <div class="demo-content">
        <p>{{ pclText('Proto.Demo.InteractionHint') }}</p>
        <div class="demo-row">
          <PButton @click="() => sideTip.show(pclText('Proto.Demo.Notification'))">{{
            pclText('Proto.Demo.ButtonDefault')
          }}</PButton>
          <PButton
            type="tint"
            @click="() => sideTip.show(pclText('Proto.Demo.Success'), 'success')"
            >{{ pclText('Proto.Demo.ButtonHighlight') }}</PButton
          >
          <PButton type="warn" @click="() => sideTip.show(pclText('Proto.Demo.Warning'), 'warn')">{{
            pclText('Proto.Demo.ButtonWarning')
          }}</PButton>
          <PButton disabled>{{ pclText('Proto.Demo.ButtonDisabled') }}</PButton>
          <PIconTextButton
            icon="lucide/heart"
            :title="pclText('Proto.Demo.Tooltip')"
            @click="() => sideTip.show(pclText('Proto.Demo.IconHint'))"
            >{{ pclText('Proto.Demo.IconButton') }}</PIconTextButton
          >
        </div>
      </div>
    </PCard>
    <PCard :title="pclText('Proto.Demo.Inputs')" default-fold-status="unfold">
      <div class="demo-content">
        <label :for="`${id}-name`">{{ pclText('Proto.Demo.PlayerName') }}</label>
        <PTextBox
          :input-id="`${id}-name`"
          v-model="playerName"
          :disabled="!editable"
          :placeholder="pclText('Proto.Demo.NamePlaceholder')"
        />
        <PComboBox
          v-model="edition"
          :options="editions"
          :aria-label="pclText('Proto.Demo.VersionType')"
        />
        <PCheckBox
          :id="`${id}-editable`"
          v-model="editable"
          :label="pclText('Proto.Demo.EnableInput')"
        />
        <PCheckBox
          :id="`${id}-disabled`"
          :model-value="true"
          disabled
          :label="pclText('Proto.Demo.DisabledCheckbox')"
        />
        <p class="demo-value">
          {{ playerName || 'Player' }} · {{ editions.find((item) => item.key === edition)?.text }}
        </p>
      </div>
    </PCard>
    <PCard :title="pclText('Proto.Demo.Loading')" default-fold-status="unfold">
      <div class="demo-content">
        <PLoading
          :state="loading"
          :card="false"
          :loading-text="pclText('Proto.Demo.LoadingText')"
        />
        <div class="demo-row">
          <PButton type="tint" @click="() => (loading = 'loading')">{{
            pclText('Proto.Demo.SetLoading')
          }}</PButton>
          <PButton type="warn" @click="() => (loading = 'error')">{{
            pclText('Proto.Demo.SetError')
          }}</PButton>
        </div>
      </div>
    </PCard>
    <PCard :title="pclText('Proto.Demo.Modal')" default-fold-status="unfold">
      <div class="demo-content">
        <p>{{ pclText('Proto.Demo.ModalHint') }}</p>
        <div class="demo-row">
          <PButton @click="() => showDialog(false)">{{
            pclText('Proto.Demo.ConfirmDialog')
          }}</PButton>
          <PButton type="tint" @click="() => showDialog(true)">{{
            pclText('Proto.Demo.InputDialog')
          }}</PButton>
        </div>
      </div>
    </PCard>
    <PCard :title="pclText('Proto.Demo.FoldCard')" default-fold-status="fold">
      <div class="demo-content">
        <PHint severity="info" closable>{{ pclText('Proto.Demo.InfoHint') }}</PHint>
        <PHint severity="warning">{{ pclText('Proto.Demo.Warning') }}</PHint>
        <PHint severity="error">{{ pclText('Proto.Demo.ErrorHint') }}</PHint>
      </div>
    </PCard>
    <PCard hide-title>
      <div class="demo-content demo-untitled">{{ pclText('Proto.Demo.UntitledCard') }}</div>
    </PCard>
  </div>
</template>

<style scoped>
.component-showcase,
.demo-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.demo-content {
  padding: 0 20px 18px;
}
.demo-untitled {
  padding-top: 18px;
}
.demo-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.demo-value {
  color: var(--color-tint);
}
</style>
