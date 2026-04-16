# GitHub Pages 部署指南

## ✅ 站点已重构完成

现代化的 bwa-rust 文档站点已构建完成，包含以下特性：

### 🌟 新特性

- **PWA 支持** - 可安装为桌面/移动应用，支持离线访问
- **双语文档** - 完整的中英文支持，带语言切换
- **现代化首页** 
  - 动态统计展示
  - 架构流程图
  - 代码对比示例
  - BWA 性能对比表格
- **增强搜索** - 本地全文搜索，带模糊匹配
- **SEO 优化** - 完整的结构化数据、Open Graph、Twitter Cards
- **响应式设计** - 移动端优化的导航和布局
- **自定义主题** - 现代化的配色和动画效果

### 📁 文件结构

```
site/
├── .vitepress/
│   ├── config.mts          # 顶级 VitePress 配置（PWA + SSR）
│   └── theme/
│       ├── index.ts        # 主题入口
│       └── style.css       # 自定义样式
├── index.md                # 中文首页
├── en/index.md             # 英文首页
├── guide/                  # 指南文档
├── api/                    # API 文档
└── public/
    ├── icons/              # PWA 图标
    └── ...
```

## 🚀 部署步骤

### 方式 1：GitHub Actions 自动部署

创建 `.github/workflows/docs.yml`：

```yaml
name: Deploy Docs

on:
  push:
    branches: [main, master]
    paths: ['docs/**', 'site/**', '.github/workflows/docs.yml']

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: false

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm

      - name: Install dependencies
        run: npm ci

      - name: Build documentation
        run: npm run docs:build

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: site/.vitepress/dist

  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

### 方式 2：本地构建并手动部署

```bash
# 1. 构建
npm run docs:build

# 2. 检查构建输出
ls site/.vitepress/dist/

# 3. 部署到 gh-pages 分支
# 使用 gh-pages 工具
npx gh-pages -d site/.vitepress/dist
```

### 方式 3：复制到 docs 分支

```bash
# 构建
npm run docs:build

# 创建临时目录
cp -r site/.vitepress/dist /tmp/bwa-rust-docs

# 切换到 gh-pages 分支
git checkout --orphan gh-pages
git rm -rf .

# 复制文件
cp -r /tmp/bwa-rust-docs/* .

# 提交
git add .
git commit -m "Update docs"
git push origin gh-pages

# 返回主分支
git checkout master
```

## 📋 本地预览

```bash
# 开发模式
npm run docs:dev

# 构建并预览
npm run docs:build
npm run docs:preview
```

## 🔧 配置说明

### PWA 配置
在 `site/.vitepress/config.mts` 中：
- `manifest` - 应用清单配置
- `workbox` - Service Worker 配置
- `icons` - 应用图标（需放置在 `public/icons/`）

### 搜索配置
```ts
search: {
  provider: 'local',
  options: {
    detailedView: true,
    // 模糊匹配
    fuzzy: 0.2,
    // 前缀匹配
    prefix: true,
  }
}
```

### SEO 配置
自动生成的 meta 标签：
- Open Graph（Facebook/LinkedIn）
- Twitter Cards
- 结构化数据（Schema.org）
- 规范链接（Canonical URLs）

## 🎨 自定义样式

修改 `site/.vitepress/theme/style.css`：

```css
:root {
  --vp-c-brand-1: #646cff;  /* 主品牌色 */
  --vp-c-brand-2: #747bff;  /* 次要品牌色 */
}
```

## 📱 PWA 安装

构建后，用户可以通过浏览器安装应用：
- Chrome: 地址栏右侧 "安装" 按钮
- Safari: 分享菜单 "添加到主屏幕"
- Edge: 设置 > 应用 > 安装此站点

## 🚨 注意事项

1. **图标**: 需要创建 PWA 图标放入 `public/icons/`
   - icon-192x192.png
   - icon-512x512.png

2. **CNAME**: 如果使用自定义域名，在 `public/CNAME` 中添加域名

3. **base 路径**: 确保 `base: '/bwa-rust/'` 与仓库名一致
