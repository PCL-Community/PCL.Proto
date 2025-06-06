<script lang="ts" setup>
import { watch } from 'vue';

export type LoadingState = 'loading' | 'error'
const props = defineProps<{ state: LoadingState }>()

watch(() => props.state, () => {
  // console.log('动画状态已切换'+ )
})

</script>

<template>
  <div id="loading-container">
    <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="80" height="80"
      viewBox="0 0 80 80" fill="none" class="load-svg" :class="state">
      <!-- 底下的横线 -->
      <path stroke="currentColor" stroke-width="2" stroke-linecap="round" d="M5 66L29 66">
      </path>
      <!-- 运动的锤子 -->
      <path class="hammer"
        d="M53.7693 59.6884L53.7763 27.5221C59.2904 27.6694 62.7075 30.1275 67.6568 31.1071C73.6227 32.4731 74.8377 33.7263 71.1429 29.9828C66.9914 25.9769 61.6417 23.1999 55.6855 22.2612C55.1971 20.2186 53.3582 18.7011 51.1671 18.7045C48.976 18.7078 47.138 20.2185 46.6487 22.2612C40.6892 23.2052 35.3415 25.9781 31.1913 29.9828C26.8344 34.236 28.2414 32.9395 34.6703 31.1142C39.6201 30.1326 43.0467 27.6687 48.5579 27.5221L48.5508 59.6884C48.5508 59.7233 48.5795 59.752 48.6145 59.752L53.7056 59.752C53.7434 59.7491 53.7663 59.7262 53.7693 59.6884Z"
        stroke="currentColor" stroke-width="2">
      </path>
      <!-- 锤子敲击后出现的三角形 -->
      <g class="triangles" fill="currentColor">
        <path d="M19.7416 60.147L21.4566 62.0039L16.8852 64.5056">
        </path>
        <path d="M11 61.67L12.8992 60.0019L15.2859 64.6345">
        </path>
      </g>
      <!-- 叉叉 -->
      <g class="wrong-x" fill="currentColor">
        <path
          d="M10.2929 49.2929C9.90237 49.6834 9.90237 50.3166 10.2929 50.7071L20.2929 60.7071C20.6834 61.0976 21.3166 61.0976 21.7071 60.7071C22.0976 60.3166 22.0976 59.6834 21.7071 59.2929L11.7071 49.2929C11.3166 48.9024 10.6834 48.9024 10.2929 49.2929Z"
          fill-rule="evenodd">
        </path>
        <path
          d="M10.2929 60.7071C10.6834 61.0976 11.3166 61.0976 11.7071 60.7071L21.7071 50.7071C22.0977 50.3166 22.0976 49.6835 21.7071 49.2929C21.3166 48.9024 20.6834 48.9024 20.2929 49.2929L10.2929 59.2929C9.9024 59.6834 9.9024 60.3166 10.2929 60.7071Z"
          fill-rule="evenodd">
        </path>
      </g>
    </svg>
  </div>
</template>

<style lang="css" scoped>
@keyframes hammerAnim {
  0% {
    transform: rotate(8deg);
  }

  40% {
    transform: rotate(-50deg);
  }

  100% {
    transform: rotate(8deg);
  }
}

.hammer {
  transform-origin: 50px 59px;
  animation: hammerAnim 2s infinite ease-in-out;
}

.load-svg.loading>.triangles {
  animation: trianglesOpacity 2s infinite ease;
}

.load-svg.loading>.wrong-x {
  opacity: 0;
}

.load-svg.error>.triangles {
  display: none;
}

@keyframes trianglesOpacity {
  0% {
    opacity: 0;
  }

  36% {
    opacity: 0;
  }

  42% {
    opacity: 0.7;
  }  100% {
    opacity: 0;
  }
}

#loading-container {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  /* background-color: antiquewhite; */
}

.load-svg {
  transition: color 0.4s ease;
}

.load-svg.loading {
  color: var(--color-titlebar);
}

.load-svg.error {
  color: var(--color-warn);
}
</style>