---
description: 文档开发 — 启动 VitePress 文档站本地开发服务器
---

启动 VitePress 文档站的本地开发服务器，用于预览和编辑文档。

// turbo
1. 安装依赖（如果 node_modules 不存在）：
```bash
npm install
```

2. 启动 VitePress 开发服务器：
```bash
npx vitepress dev site
```

3. 打开浏览器预览文档站。

4. 文档结构说明：
   - `site/index.md` — 首页
   - `site/guide/getting-started.md` — 快速开始
   - `site/guide/architecture.md` — 架构文档
   - `site/guide/tutorial.md` — 教程
   - `site/.vitepress/config.mts` — VitePress 配置
