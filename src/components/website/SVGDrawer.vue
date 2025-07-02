<script lang="ts" setup>
import { ref, onMounted } from 'vue'
import { h, render } from 'vue'

// 存储 SVG 图标组件信息
const svgIconComponents = ref<{ name: string; component: any; isVueComponent: boolean }[]>([])
// 控制抽屉展开状态
const isOpen = ref(false)

// 切换抽屉展开状态
const toggleDrawer = () => {
  isOpen.value = !isOpen.value
}

// 下载 SVG 图标
const downloadSvg = async (name: string, content: string | any) => {
  let svgContent: string
  if (typeof content === 'string') {
    try {
      const response = await fetch(content)
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }
      svgContent = await response.text()
    } catch (error) {
      console.error('Failed to fetch SVG content:', error)
      return
    }
  } else {
    const div = document.createElement('div')
    // 手动渲染组件
    const instance = h(content)
    render(instance, div)
    const svgElement = div.querySelector('svg')
    if (svgElement) {
      svgContent = new XMLSerializer().serializeToString(svgElement)
    } else {
      return
    }
  }
  const blob = new Blob([svgContent], { type: 'image/svg+xml' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `${name}.svg`
  link.click()
  URL.revokeObjectURL(url)
}

onMounted(async () => {
  // 使用 Vite 的动态导入功能获取图标组件
  const vueIconModules = import.meta.glob('@/components/icons/**/*.vue', { eager: true })
  for (const path in vueIconModules) {
    const module = vueIconModules[path] as { default: any }
    const name = path.split('/').pop()?.replace('.vue', '')
    if (name && module.default) {
      svgIconComponents.value.push({ name, component: module.default, isVueComponent: true })
    }
  }

  // 使用 Vite 的动态导入功能获取 SVG 文件
  const svgFileModules = import.meta.glob('@/assets/icons/**/*.svg', {
    query: '?url',
    import: 'default',
    eager: true,
  })
  for (const path in svgFileModules) {
    const module = svgFileModules[path] as string
    const name = path.split('/').pop()?.replace('.svg', '')
    if (name && module) {
      svgIconComponents.value.push({ name, component: module, isVueComponent: false })
    }
  }
})
</script>

<template>
  <div class="drawer-container">
    <!-- 抽屉触发按钮 -->
    <button class="drawer-toggle" @click="toggleDrawer">
      {{ isOpen ? '收起SVG图标库' : '展开SVG图标库' }}
    </button>
    <!-- 抽屉内容 -->
    <div class="drawer" :class="{ open: isOpen }">
      <div class="drawer-content">
        <!-- 以网格布局展示 SVG 图标 -->
        <div class="svg-grid">
          <div v-for="icon in svgIconComponents" :key="icon.name" class="svg-item">
            <div v-if="icon.isVueComponent">
              <component :is="icon.component" @click="downloadSvg(icon.name, icon.component)" />
            </div>
            <div v-else>
              <img
                :src="icon.component"
                alt="icon.name"
                @click="downloadSvg(icon.name, icon.component)"
              />
            </div>
            <div class="svg-name">{{ icon.name }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="css" scoped>
.drawer-container {
  position: fixed;
  bottom: 0;
  left: 0;
  width: 100%;
  z-index: 1000;
}

.drawer-toggle {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  background-color: #007bff;
  color: white;
  border: none;
  padding: 8px 16px;
  cursor: pointer;
  border-radius: 5px 5px 0 0;
}

.drawer {
  background-color: white;
  height: 0;
  overflow: hidden;
  transition: height 0.3s ease;
  box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.1);
}

.drawer.open {
  height: 50vh; /* 展开时占屏幕一半高度 */
}

.drawer-content {
  height: 100%;
  overflow-y: auto; /* 内容溢出时可滚动 */
  padding: 16px;
}

.svg-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  gap: 16px;
}

.svg-item {
  text-align: center;
  cursor: pointer;
}

.svg-item img {
  width: 100%;
  height: 80px;
  background-color: #f0f0f0;
  border-radius: 5px;
  padding: 10px;
  object-fit: contain;
}

.svg-item svg {
  width: 100%;
  height: 80px;
  background-color: #f0f0f0;
  border-radius: 5px;
  padding: 10px;
}

.svg-name {
  font-size: 12px;
  color: #333;
}
</style>
