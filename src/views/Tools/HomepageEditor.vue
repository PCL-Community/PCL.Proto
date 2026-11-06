<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import type { Element } from 'xml-js'
import PCard from '@/components/widget/PCard.vue'
import PButton from '@/components/widget/PButton.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PCustomHomepage from '@/components/widget/PCustomHomepage.vue'
import { pclText } from '@/api/pclLocale'
import { readLocalHomepage, saveLocalHomepage } from '@/api/customHomepage'
import { controlName, homepageLimit, parseHomepage, serializeHomepage } from '@/util/homepageXaml'
import template from '@/assets/homepage/Custom.xaml?raw'
import sideTip from '@/composables/sideTip'

const draftKey = 'pcl-homepage:editor-draft'
const source = ref(template)
const error = ref<string>()
const busy = ref(false)
const sourceMode = ref(false)
const selectedPath = ref('0')
const addType = ref('TextBlock')
const fileInput = ref<HTMLInputElement>()
try {
  source.value = localStorage.getItem(draftKey) ?? template
} catch (failure) {
  error.value = String(failure)
}
const parsed = computed(() => {
  try {
    return { ...parseHomepage(source.value), error: undefined }
  } catch (failure) {
    return { nodes: [] as Element[], warnings: [], error: String(failure) }
  }
})
const tree = computed(() => {
  const entries: { path: string; node: Element; depth: number }[] = []
  function visit(nodes: Element[], parent: string, depth: number) {
    nodes.forEach((node, index) => {
      const path = parent ? `${parent}.${index}` : String(index)
      if (node.type !== 'element' || node.name?.includes('.')) return
      entries.push({ path, node, depth })
      if (node.elements) visit(node.elements, path, depth + 1)
    })
  }
  visit(parsed.value.nodes, '', 0)
  return entries
})
const selected = computed(() => tree.value.find((entry) => entry.path === selectedPath.value))
watch(tree, (entries) => {
  if (!entries.some((entry) => entry.path === selectedPath.value))
    selectedPath.value = entries[0]?.path ?? '0'
})
function componentLabel(node: Element) {
  const name = controlName(node)
  return components.includes(name) ? pclText(`Proto.Homepage.Control.${name}`) : name
}
function componentSummary(node: Element) {
  return String(
    node.attributes?.Title || node.attributes?.Text || node.attributes?.Source || controlName(node),
  )
}
async function navigateComponents(event: KeyboardEvent) {
  const index = tree.value.findIndex((entry) => entry.path === selectedPath.value)
  let target = index
  if (event.key === 'ArrowDown') target = Math.min(index + 1, tree.value.length - 1)
  else if (event.key === 'ArrowUp') target = Math.max(index - 1, 0)
  else if (event.key === 'Home') target = 0
  else if (event.key === 'End') target = tree.value.length - 1
  else return
  event.preventDefault()
  const entry = tree.value[target]
  if (!entry) return
  const list = event.currentTarget as HTMLElement
  selectedPath.value = entry.path
  await nextTick()
  const row = list.querySelector<HTMLElement>('[tabindex="0"]')
  row?.focus({ preventScroll: true })
  row?.scrollIntoView({ block: 'nearest' })
}
const fields = computed(() => {
  if (!selected.value) return []
  const node = selected.value.node
  const name = controlName(node)
  const primary =
    name === 'MyCard'
      ? ['Title', 'CanSwap', 'IsSwaped']
      : name === 'MyListItem'
        ? ['Title', 'Info', 'Logo', 'EventType', 'EventData']
        : name === 'Image' || name === 'MyImage'
          ? ['Source', 'FallbackSource', 'Height']
          : name === 'MyHint'
            ? ['Text', 'Theme']
            : name.includes('Button')
              ? ['Text', 'ColorType', 'EventType', 'EventData', 'ToolTip']
              : name === 'TextBlock'
                ? ['Text', 'FontSize', 'Foreground', 'FontWeight']
                : name === 'StackPanel' || name === 'WrapPanel'
                  ? ['Orientation']
                  : []
  return [
    ...new Set([...primary, 'Margin', 'Width', 'Height', ...Object.keys(node.attributes || {})]),
  ]
})
const components = [
  'MyCard',
  'TextBlock',
  'MyHint',
  'MyButton',
  'MyListItem',
  'MyImage',
  'StackPanel',
  'WrapPanel',
  'Grid',
]
let draftTimer: ReturnType<typeof setTimeout> | undefined
function persistDraft() {
  try {
    localStorage.setItem(draftKey, source.value)
  } catch (failure) {
    error.value = String(failure)
  }
}
watch(source, () => {
  clearTimeout(draftTimer)
  draftTimer = setTimeout(persistDraft, 300)
})
onBeforeUnmount(() => {
  clearTimeout(draftTimer)
  persistDraft()
})
function commitTree() {
  source.value = serializeHomepage(parsed.value.nodes)
}
function setAttribute(key: string, value: string | undefined) {
  const node = selected.value?.node
  if (!node) return
  node.attributes ||= {}
  if (value === undefined || value === '') delete node.attributes[key]
  else node.attributes[key] = value
  commitTree()
}
function add() {
  if (parsed.value.error) return
  const name = addType.value
  const node: Element = {
    type: 'element',
    name: name.startsWith('My') ? `local:${name}` : name,
    attributes: {},
  }
  if (name === 'MyCard') {
    node.attributes = { Title: pclText('Proto.Homepage.NewCard'), Margin: '0,0,0,15' }
    node.elements = [
      { type: 'element', name: 'StackPanel', attributes: { Margin: '25,40,23,15' }, elements: [] },
    ]
  } else if (name === 'MyListItem')
    node.attributes = {
      Title: pclText('Proto.Homepage.NewText'),
      Info: pclText('Proto.Homepage.NewText'),
    }
  else if (name === 'MyImage')
    node.attributes = { Source: 'https://www.minecraft.net/favicon.ico', Height: '64' }
  else if (!['StackPanel', 'WrapPanel', 'Grid'].includes(name))
    node.attributes = { Text: pclText('Proto.Homepage.NewText'), Margin: '0,0,0,10' }
  if (name === 'MyHint') node.attributes!.Theme = 'Blue'
  if (name === 'MyButton')
    Object.assign(node.attributes!, {
      EventType: '打开网页',
      EventData: 'https://www.minecraft.net/',
    })
  let parent = name === 'MyCard' ? undefined : selected.value
  while (
    parent &&
    !['MyCard', 'StackPanel', 'WrapPanel', 'Grid', 'Border'].includes(controlName(parent.node))
  ) {
    parent = tree.value.find(
      (entry) => entry.path === parent!.path.split('.').slice(0, -1).join('.'),
    )
  }
  if (parent && controlName(parent.node) === 'MyCard') {
    parent =
      tree.value.find(
        (entry) => entry.path === `${parent!.path}.0` && controlName(entry.node) === 'StackPanel',
      ) || parent
  }
  const siblings = parent ? (parent.node.elements ||= []) : parsed.value.nodes
  siblings.push(node)
  selectedPath.value = parent
    ? `${parent.path}.${siblings.length - 1}`
    : String(siblings.length - 1)
  commitTree()
}
function siblingsOfSelection() {
  const parts = selectedPath.value.split('.').map(Number)
  let nodes = parsed.value.nodes
  for (const index of parts.slice(0, -1)) nodes = nodes[index]?.elements || []
  return { nodes, index: parts.at(-1)! }
}
function moveTarget(direction: number) {
  const { nodes, index } = siblingsOfSelection()
  for (let target = index + direction; target >= 0 && target < nodes.length; target += direction) {
    if (nodes[target]?.type === 'element' && !nodes[target]?.name?.includes('.')) return target
  }
  return undefined
}
function move(direction: number) {
  const { nodes, index } = siblingsOfSelection()
  const target = moveTarget(direction)
  if (target === undefined) return
  ;[nodes[index], nodes[target]] = [nodes[target]!, nodes[index]!]
  selectedPath.value = [...selectedPath.value.split('.').slice(0, -1), target].join('.')
  commitTree()
}
function remove() {
  const { nodes, index } = siblingsOfSelection()
  const parent = selectedPath.value.split('.').slice(0, -1).join('.')
  nodes.splice(index, 1)
  const sibling = nodes.findIndex(
    (node, i) => i >= index && node.type === 'element' && !node.name?.includes('.'),
  )
  selectedPath.value = sibling >= 0 ? [parent, sibling].filter(String).join('.') : parent || '0'
  commitTree()
}
async function importFile(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  try {
    if (file.size > homepageLimit) throw new Error('Homepage exceeds 2 MiB')
    const content = new TextDecoder('utf-8', { fatal: true }).decode(await file.arrayBuffer())
    parseHomepage(content)
    source.value = content
    selectedPath.value = '0'
    error.value = undefined
  } catch (failure) {
    error.value = String(failure)
  } finally {
    input.value = ''
  }
}
function exportFile() {
  const url = URL.createObjectURL(
    new Blob([source.value], { type: 'application/xml;charset=utf-8' }),
  )
  const link = document.createElement('a')
  link.href = url
  link.download = 'Custom.xaml'
  link.click()
  setTimeout(() => URL.revokeObjectURL(url), 1000)
}
async function load() {
  const previous = source.value
  busy.value = true
  try {
    const content = await readLocalHomepage()
    parseHomepage(content)
    if (source.value === previous) source.value = content
    error.value = undefined
  } catch (failure) {
    error.value = String(failure)
  } finally {
    busy.value = false
  }
}
async function apply() {
  busy.value = true
  try {
    await saveLocalHomepage(source.value)
    error.value = undefined
    sideTip.show(pclText('Proto.Homepage.Applied'), 'success')
  } catch (failure) {
    error.value = String(failure)
  } finally {
    busy.value = false
  }
}
</script>
<template>
  <section class="pcl-page homepage-editor">
    <PCard :title="pclText('Proto.Title.HomepageEditor')">
      <div class="editor-controls">
        <div class="editor-actions">
          <PButton @click="fileInput?.click()">{{ pclText('Proto.Homepage.Import') }}</PButton>
          <PButton @click="exportFile" :disabled="Boolean(parsed.error)">{{
            pclText('Proto.Homepage.Export')
          }}</PButton>
          <PButton @click="load" :disabled="busy">{{ pclText('Proto.Homepage.Load') }}</PButton>
          <PButton type="tint" @click="apply" :disabled="busy || Boolean(parsed.error)">{{
            pclText('Proto.Homepage.Apply')
          }}</PButton>
          <input
            ref="fileInput"
            type="file"
            accept=".xaml,.xml,text/xml,application/xml"
            hidden
            @change="importFile"
          />
        </div>
        <PHint severity="info">{{ pclText('Proto.Homepage.EditorHint') }}</PHint>
        <PHint v-if="error" severity="error">{{ error }}</PHint>
      </div>
    </PCard>
    <div class="editor-columns">
      <PCard :title="pclText('Proto.Homepage.Edit')">
        <div class="editor-controls">
          <div class="editor-modes" :aria-label="pclText('Proto.Homepage.Edit')">
            <button type="button" :aria-pressed="!sourceMode" @click="sourceMode = false">
              {{ pclText('Proto.Homepage.Visual') }}
            </button>
            <button type="button" :aria-pressed="sourceMode" @click="sourceMode = true">
              XAML
            </button>
          </div>
          <PTextBox
            v-if="sourceMode"
            v-model="source"
            multiline
            class="xaml-source"
            aria-label="XAML"
            spellcheck="false"
          />
          <template v-else>
            <div class="editor-add">
              <PComboBox
                v-model="addType"
                :options="
                  components.map((key) => ({ key, text: pclText(`Proto.Homepage.Control.${key}`) }))
                "
                :aria-label="pclText('Proto.Homepage.Components')"
              />
              <PButton compact type="tint" @click="add" :disabled="Boolean(parsed.error)">{{
                pclText('Proto.Homepage.Add')
              }}</PButton>
            </div>
            <PHint v-if="parsed.error" severity="error">{{ parsed.error }}</PHint>
            <div class="outline-heading">
              <span>{{ pclText('Proto.Homepage.Components') }}</span>
              <span class="component-count">{{ tree.length }}</span>
            </div>
            <div
              v-if="tree.length"
              class="component-tree"
              role="listbox"
              :aria-label="pclText('Proto.Homepage.Components')"
              @keydown="navigateComponents"
            >
              <div
                v-for="entry in tree"
                :key="entry.path"
                class="component-row"
                role="option"
                :aria-selected="selectedPath === entry.path"
                :tabindex="selectedPath === entry.path ? 0 : -1"
                :style="{ paddingInlineStart: `${12 + entry.depth * 16}px` }"
                :title="`${controlName(entry.node)} · ${componentSummary(entry.node)}`"
                @focus="selectedPath = entry.path"
                @click="selectedPath = entry.path"
              >
                <span class="component-branch" aria-hidden="true">{{
                  entry.depth ? '└' : '◇'
                }}</span>
                <span class="component-description">
                  <span class="component-summary">{{ componentSummary(entry.node) }}</span>
                  <span class="component-type">{{ componentLabel(entry.node) }}</span>
                </span>
              </div>
            </div>
            <p v-else-if="!parsed.error" class="editor-empty">
              {{ pclText('Proto.Homepage.Empty') }}
            </p>
            <template v-if="selected">
              <div class="inspector-heading">
                <div>
                  <p>{{ pclText('Proto.Homepage.Properties') }}</p>
                  <span class="component-type">{{ controlName(selected.node) }}</span>
                </div>
                <div class="editor-actions">
                  <button
                    type="button"
                    class="inspector-action"
                    @click="move(-1)"
                    :disabled="moveTarget(-1) === undefined"
                    :title="pclText('Proto.Homepage.Up')"
                    :aria-label="pclText('Proto.Homepage.Up')"
                  >
                    ↑
                  </button>
                  <button
                    type="button"
                    class="inspector-action"
                    @click="move(1)"
                    :disabled="moveTarget(1) === undefined"
                    :title="pclText('Proto.Homepage.Down')"
                    :aria-label="pclText('Proto.Homepage.Down')"
                  >
                    ↓
                  </button>
                  <button type="button" class="inspector-action delete-action" @click="remove">
                    {{ pclText('Common.Action.Delete') }}
                  </button>
                </div>
              </div>
              <div v-for="key in fields" :key="key" class="editor-field">
                <label :for="`homepage-${key}`">{{ key }}</label>
                <PTextBox
                  :input-id="`homepage-${key}`"
                  :model-value="String(selected.node.attributes?.[key] ?? '')"
                  :multiline="['Text', 'Info', 'ToolTip'].includes(key)"
                  @update:model-value="setAttribute(key, $event)"
                />
              </div>
            </template>
          </template>
        </div>
      </PCard>
      <div class="editor-preview">
        <p class="preview-title">{{ pclText('Proto.Homepage.Preview') }}</p>
        <PCustomHomepage :source="source" preview />
      </div>
    </div>
  </section>
</template>
<style scoped>
.homepage-editor {
  container-type: inline-size;
  height: 100%;
  min-height: 0;
  overflow-y: auto;
}
.homepage-editor > * {
  flex-shrink: 0;
}
.editor-controls {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 0 25px 20px;
}
.editor-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
.editor-columns {
  display: grid;
  grid-template-columns: minmax(280px, 0.85fr) minmax(0, 1.15fr);
  gap: 20px;
  align-items: start;
}
.editor-add {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
}
.component-tree {
  max-height: 260px;
  overflow: auto;
  border: 1px solid var(--ce-gray-5);
  border-radius: 4px;
  padding: 4px;
}
.component-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 48px;
  padding: 6px 10px;
  border-radius: 3px;
  cursor: pointer;
  position: relative;
  transition:
    background-color 100ms,
    color 100ms;
}
.component-row:hover,
.inspector-action:not(:disabled):hover {
  background: var(--color-tint-lighter);
}
.component-row[aria-selected='true'] {
  background: var(--ce-color-7);
  color: var(--color-tint);
}
.component-row[aria-selected='true']::before {
  content: '';
  position: absolute;
  left: 0;
  width: 3px;
  height: 22px;
  border-radius: 2px;
  background: var(--color-tint);
}
.component-row:focus-visible,
.editor-modes button:focus-visible,
.inspector-action:focus-visible {
  outline: 2px solid var(--color-tint);
  outline-offset: -2px;
}
.component-branch {
  flex: 0 0 12px;
  color: var(--color-text-grey);
}
.component-description {
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.component-summary {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.component-type,
.component-count,
.editor-empty {
  color: var(--color-text-grey);
  font-size: 12px;
}
.outline-heading,
.inspector-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.outline-heading {
  margin-bottom: -6px;
}
.inspector-heading {
  border-top: 1px solid var(--ce-gray-5);
  padding-top: 16px;
  margin-top: 4px;
}
.inspector-heading p {
  margin: 0;
  font-weight: 600;
}
.inspector-heading .editor-actions {
  gap: 3px;
}
.inspector-action,
.editor-modes button {
  font: inherit;
  color: inherit;
  border: 0;
  background: transparent;
  cursor: pointer;
  border-radius: 3px;
  padding: 5px 9px;
}
.inspector-action:disabled {
  opacity: 0.35;
  cursor: default;
}
.delete-action {
  color: var(--color-text-grey);
}
.delete-action:hover {
  color: var(--color-warn, #c74646);
}
.editor-modes {
  display: flex;
  border-bottom: 1px solid var(--ce-gray-5);
  gap: 20px;
}
.editor-modes button {
  padding: 6px 0 10px;
  border-radius: 0;
  border-bottom: 2px solid transparent;
  color: var(--color-text-grey);
}
.editor-modes button[aria-pressed='true'] {
  border-bottom-color: var(--color-tint);
  color: var(--color-tint);
}
.editor-field {
  display: grid;
  grid-template-columns: 100px minmax(0, 1fr);
  gap: 10px;
  align-items: center;
}
.editor-field label {
  overflow-wrap: anywhere;
  font-size: 12px;
}
.xaml-source {
  height: 550px;
  font-family: ui-monospace, monospace;
}
.editor-preview {
  min-width: 0;
  position: sticky;
  top: 0;
}
.preview-title {
  margin: 0 0 12px;
  color: var(--color-text-grey);
}
@container (max-width: 680px) {
  .editor-columns {
    grid-template-columns: minmax(0, 1fr);
  }
  .editor-preview {
    position: static;
  }
}
</style>
