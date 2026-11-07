<template>
  <div class="list-actions" @click.stop><slot /></div>
</template>

<style scoped>
.list-actions {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  gap: 0;
  max-width: 0;
  overflow: hidden;
  opacity: 0;
  pointer-events: none;
  transition: opacity 72ms;
}
:global(.gameinfo-container:hover .list-actions),
:global(.gameinfo-container:focus-within .list-actions),
:global(.gameinfo-container .list-actions:has(details[open])),
:global(.comp-item:hover .list-actions),
:global(.comp-item:focus-within .list-actions) {
  max-width: none;
  overflow: visible;
  opacity: 1;
  pointer-events: auto;
  transition-duration: 84ms;
  transition-delay: 36ms;
}
@media (hover: none) {
  .list-actions {
    max-width: none;
    overflow: visible;
    opacity: 1;
    pointer-events: auto;
  }
}
@media (prefers-reduced-motion: reduce) {
  .list-actions {
    transition: none;
  }
}
</style>
