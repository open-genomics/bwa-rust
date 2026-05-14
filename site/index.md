---
layout: home
---

<script setup>
import { onMounted } from 'vue'
import { useData } from 'vitepress'

const { base } = useData()

onMounted(() => {
  const lang = navigator.language
  // 使用 base 前缀确保路径正确
  location.href = lang.startsWith('zh') ? `${base}zh/` : `${base}en/`
})
</script>

<head>
  <!-- 使用相对路径，meta refresh 不支持 JS 变量 -->
  <meta http-equiv="refresh" content="0;url=./en/">
</head>
