---
layout: home
---

<head>
  <meta http-equiv="refresh" content="0;url=/en/">
</head>

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
