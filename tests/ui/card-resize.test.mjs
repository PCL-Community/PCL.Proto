import assert from 'node:assert/strict'

let observer
let reducedMotion = false
let motionChanged
globalThis.window = {
  matchMedia: () => ({
    get matches() {
      return reducedMotion
    },
    addEventListener(_event, callback) {
      motionChanged = callback
    },
  }),
}
globalThis.ResizeObserver = class {
  constructor(callback) {
    this.callback = callback
    this.targets = new Set()
    observer = this
  }
  observe(target) {
    this.targets.add(target)
  }
  unobserve(target) {
    this.targets.delete(target)
  }
  notify(...targets) {
    this.callback(targets.map((target) => ({ target })))
  }
}
const { observeCardResize } = await import('../../src/util/cardResize.ts')
const inner = {}
const animations = []
let naturalHeight = 80
let displayedHeight
const card = {
  get offsetHeight() {
    return displayedHeight ?? naturalHeight
  },
  contains() {
    return false
  },
  animate(frames, options) {
    const animation = {
      frames,
      options,
      cancel() {
        displayedHeight = undefined
        this.cancelled = true
      },
    }
    animations.push(animation)
    return animation
  },
}
const stop = observeCardResize(inner, card)
observer.notify(inner)
assert.equal(animations.length, 0, 'initial sizing must not animate')
naturalHeight = 160
observer.notify(inner)
assert.deepEqual(animations[0].frames.map((frame) => frame.height), ['80px', '164px', '160px'])
displayedHeight = 120
naturalHeight = 100
observer.notify(inner)
assert.deepEqual(animations[1].frames.map((frame) => frame.height), ['120px', '98px', '100px'])
animations[1].onfinish()
naturalHeight = 140
observer.notify(inner)
reducedMotion = true
motionChanged()
assert.equal(animations[2].cancelled, true, 'enabling reduced motion cancels an active animation')
naturalHeight = 200
observer.notify(inner)
assert.equal(animations.length, 3, 'reduced motion must not animate')
stop()
assert.equal(observer.targets.size, 0, 'unmount must stop observing')
reducedMotion = false
const childInner = {}
const parentInner = {}
let childHeight = 50
let parentHeight = 100
const child = {
  get offsetHeight() { return childHeight },
  contains() { return false },
  animate: card.animate,
}
const parent = {
  get offsetHeight() { return parentHeight },
  contains(element) { return element === child },
  animate: card.animate,
}
const stopParent = observeCardResize(parentInner, parent)
const stopChild = observeCardResize(childInner, child)
observer.notify(parentInner, childInner)
childHeight = 90
parentHeight = 140
observer.notify(parentInner, childInner)
assert.equal(animations.length, 4, 'nested changes animate only the child')
animations[3].onfinish()
stopParent()
stopChild()
console.log('Card resize animation, interruption, reduced motion and cleanup passed.')
