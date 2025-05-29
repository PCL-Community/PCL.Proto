<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import TitleMessage from './components/TitleMessage.vue'
import { ref } from 'vue';

const navWidth = ref('220px')
</script>

<template>
  <!-- 图文介绍的标题部分 -->
  <TitleMessage />

  <!-- 以下为主体部分 -->
  <main>
    <header>
      <nav>
        <RouterLink to="/">启动</RouterLink>
        <RouterLink to="/download">下载</RouterLink>
        <RouterLink to="/link">联机</RouterLink>
        <RouterLink to="/setup">设置</RouterLink>
      </nav>
    </header>
    <RouterView />
  </main>
</template>

<style scoped>
nav {
  width: 100%;
  margin-top: 2rem;
  text-align: left;
  margin-left: -1rem;
  font-size: 1rem;

  padding: 1rem 0;
  margin-top: 1rem;
}

nav a.router-link-exact-active {
  color: var(--color-text);
}

nav a.router-link-exact-active:hover {
  background-color: transparent;
}

nav a {
  display: inline-block;
  padding: 0 1rem;
  border-left: 1px solid var(--color-border);
}

nav a:first-of-type {
  border: 0;
}
</style>
