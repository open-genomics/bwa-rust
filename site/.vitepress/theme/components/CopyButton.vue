<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  content: string
}>()

const copied = ref(false)

const copy = async () => {
  try {
    await navigator.clipboard.writeText(props.content)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}
</script>

<template>
  <button 
    class="copy-btn"
    @click="copy"
    :class="{ copied }"
  >
    <span v-if="copied">✅ Copied!</span>
    <span v-else>📋 Copy</span>
  </button>
</template>

<style scoped>
.copy-btn {
  padding: 0.5rem 1rem;
  border-radius: 8px;
  border: 1px solid var(--vp-c-border);
  background: var(--vp-c-bg);
  color: var(--vp-c-text-1);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s;
}

.copy-btn:hover {
  background: var(--vp-c-brand-soft);
  border-color: var(--vp-c-brand-1);
}

.copy-btn.copied {
  background: var(--vp-c-success);
  border-color: var(--vp-c-success);
  color: white;
}
</style>
