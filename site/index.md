---
layout: home
---

<script setup>
import { onMounted } from 'vue'
onMounted(() => {
  const lang = navigator.language
  if (lang.startsWith('zh')) {
    location.href = '/zh/'
  } else {
    location.href = '/en/'
  }
})
</script>
