<script setup lang="ts">
import DOMPurify from 'dompurify'
import { marked } from 'marked'
import { computed } from 'vue'

const props = defineProps<{ content: string }>()
const html = computed(() => {
  const fragment = DOMPurify.sanitize(marked.parse(props.content, { async: false }), {
    ADD_TAGS: ['iframe'],
    RETURN_DOM_FRAGMENT: true,
    FORBID_TAGS: ['style'],
  })
  for (const frame of fragment.querySelectorAll('iframe')) {
    try {
      const url = new URL(frame.getAttribute('src') || '', window.location.href)
      if (url.protocol !== 'https:') throw new Error('Unsupported iframe URL')
      frame.src = url.href
      frame.removeAttribute('srcdoc')
      frame.setAttribute('sandbox', 'allow-scripts allow-presentation')
      frame.setAttribute('loading', 'lazy')
    } catch {
      frame.remove()
    }
  }
  const container = document.createElement('div')
  container.append(fragment)
  return container.innerHTML
})
</script>

<template><div class="markdown" v-html="html" /></template>

<style scoped>
.markdown {
  min-width: 0;
  overflow-wrap: anywhere;
  user-select: text;
  line-height: 1.6;
}
.markdown :deep(> :first-child) {
  margin-top: 0;
}
.markdown :deep(> :last-child) {
  margin-bottom: 0;
}
.markdown :deep(h1),
.markdown :deep(h2),
.markdown :deep(h3) {
  line-height: 1.3;
  margin: 1em 0 0.5em;
}
.markdown :deep(p),
.markdown :deep(ul),
.markdown :deep(ol),
.markdown :deep(pre),
.markdown :deep(blockquote) {
  margin: 0.7em 0;
}
.markdown :deep(ul),
.markdown :deep(ol) {
  padding-left: 1.8em;
}
.markdown :deep(img) {
  max-width: 100%;
  height: auto;
}
.markdown :deep(iframe),
.markdown :deep(video) {
  max-width: 100%;
}
.markdown :deep(pre) {
  overflow-x: auto;
  padding: 10px;
  background: var(--color-background);
}
.markdown :deep(blockquote) {
  border-left: 3px solid var(--ce-color-2);
  padding-left: 12px;
}
.markdown :deep(a) {
  color: var(--ce-color-2);
}
.markdown :deep(table) {
  display: block;
  max-width: 100%;
  overflow-x: auto;
  border-collapse: collapse;
}
.markdown :deep(th),
.markdown :deep(td) {
  border: 1px solid var(--color-text-gray);
  padding: 4px 8px;
}
</style>
