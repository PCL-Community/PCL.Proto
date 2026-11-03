type CardState = {
  card: HTMLElement
  height?: number
  animation?: Animation
}

const cards = new WeakMap<Element, CardState>()
const animating = new Set<CardState>()
let observer: ResizeObserver | undefined
let reducedMotion: MediaQueryList | undefined

function motionPreference() {
  if (!reducedMotion) {
    reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)')
    reducedMotion.addEventListener('change', () => {
      if (!reducedMotion?.matches) return
      for (const state of animating) {
        state.animation?.cancel()
        state.animation = undefined
      }
      animating.clear()
    })
  }
  return reducedMotion
}

function resizeCards(entries: ResizeObserverEntry[]) {
  const writes: (() => void)[] = []
  const scheduled = new Set<CardState>()
  const changed = new Map<CardState, boolean>()
  for (const entry of entries) {
    const state = cards.get(entry.target)
    if (!state) continue
    changed.set(state, changed.get(state) || entry.target !== state.card)
  }
  const ordered = [...changed].sort(([a], [b]) =>
    a.card.contains(b.card) ? 1 : b.card.contains(a.card) ? -1 : 0,
  )
  for (const [state, innerChanged] of ordered) {
    const { card } = state
    if (state.animation && !innerChanged) continue
    const from = state.animation ? card.offsetHeight : state.height
    state.animation?.cancel()
    state.animation = undefined
    animating.delete(state)
    const height = card.offsetHeight
    state.height = height
    if (
      from === undefined ||
      from === 0 ||
      height === 0 ||
      from === height ||
      motionPreference().matches ||
      [...animating, ...scheduled].some((child) => card.contains(child.card))
    )
      continue
    scheduled.add(state)
    writes.push(() => {
      const overshoot = Math.min(Math.abs(height - from) * 0.1, 4, height)
      const animation = card.animate([
        { height: `${from}px`, easing: 'cubic-bezier(0.22, 1, 0.36, 1)' },
        { height: `${height + Math.sign(height - from) * overshoot}px`, offset: 0.75, easing: 'ease-in-out' },
        { height: `${height}px` },
      ], {
        duration: 400,
      })
      state.animation = animation
      animating.add(state)
      animation.onfinish = () => {
        if (state.animation === animation) {
          state.animation = undefined
          animating.delete(state)
        }
      }
    })
  }
  writes.forEach((write) => write())
}

export function observeCardResize(inner: HTMLElement, card: HTMLElement) {
  observer ||= new ResizeObserver(resizeCards)
  const state: CardState = { card }
  cards.set(inner, state)
  cards.set(card, state)
  observer.observe(inner)
  observer.observe(card)
  return () => {
    observer?.unobserve(inner)
    observer?.unobserve(card)
    cards.delete(inner)
    cards.delete(card)
    state.animation?.cancel()
    animating.delete(state)
  }
}
