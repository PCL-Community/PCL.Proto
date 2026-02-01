import { defineStore } from 'pinia'

export const defaultWidths = {
  home: 286,
  download: 139,
  link: 120,
  setup: 120,
  more: 120,
  task_manage: 200,
}

const useSideNavState = defineStore('side-nav-state', {
  state: () => ({
    isShown: true,
    width: defaultWidths.home,
  }),
  actions: {
    setWidth(width: number) {
      this.width = width
    },
    setWidthOfPageDefault(page: keyof typeof defaultWidths) {
      this.width = defaultWidths[page]
    },
  },
  getters: {
    sideNavWidthStr: (state) => state.width + 'px',
  },
})

export default useSideNavState
