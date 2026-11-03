<template>
  <svg class="selection-indicator" width="4" height="23" viewBox="0 0 4 23" aria-hidden="true">
    <line
      x1="2"
      y1="2"
      x2="2"
      y2="21"
      stroke="currentColor"
      stroke-width="4"
      stroke-linecap="round"
    />
  </svg>
</template>

<style scoped>
.selection-indicator {
  flex-shrink: 0;
  color: var(--color-tint);
  visibility: hidden;
  opacity: 0;
  transform: scaleY(0);
  transform-origin: center;
  /* CE MyListItem：选中条 300 ms 回弹展开，取消 120 ms 收起 + 70 ms 淡出。 */
  transition:
    transform 120ms cubic-bezier(0.55, 0, 1, 0.45),
    opacity 70ms linear 40ms,
    visibility 0s linear 120ms;
}
:global(a.router-link-active > .selection-indicator),
:global(.gameinfo-container[aria-pressed='true'] > .selection-indicator) {
  visibility: visible;
  opacity: 1;
  transform: scaleY(1);
  transition:
    transform 300ms cubic-bezier(0.34, 1.56, 0.64, 1),
    opacity 0s linear 0s,
    visibility 0s linear 0s;
}
@media (prefers-reduced-motion: reduce) {
  .selection-indicator,
  :global(a.router-link-active > .selection-indicator),
  :global(.gameinfo-container[aria-pressed='true'] > .selection-indicator) {
    transition: none;
  }
}
</style>
