<script setup lang="ts">
import { useData } from 'vitepress'
import { computed } from 'vue'

const { site, localeIndex, lang } = useData()

const currentLang = computed(() => {
  return localeIndex.value === 'root' ? '中文' : 'EN'
})

const switchLang = () => {
  const currentPath = window.location.pathname
  const isZh = localeIndex.value === 'root'
  
  if (isZh) {
    // Switch to English
    window.location.href = currentPath.replace('/bwa-rust/', '/bwa-rust/en/')
  } else {
    // Switch to Chinese
    window.location.href = currentPath.replace('/bwa-rust/en/', '/bwa-rust/')
  }
}
</script>

<template>
  <button 
    class="lang-switch"
    @click="switchLang"
    :title="currentLang === '中文' ? 'Switch to English' : '切换到中文'"
  >
    <span class="lang-icon">🌐</span>
    <span class="lang-text">{{ currentLang }}</span>
  </button>
</template>

<style scoped>
.lang-switch {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border-radius: 8px;
  border: 1px solid var(--vp-c-border);
  background: var(--vp-c-bg);
  color: var(--vp-c-text-1);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.lang-switch:hover {
  background: var(--vp-c-brand-soft);
  border-color: var(--vp-c-brand-1);
}

.lang-icon {
  font-size: 1rem;
}

.lang-text {
  min-width: 2rem;
}

@media (max-width: 768px) {
  .lang-text {
    display: none;
  }
}
</style>
