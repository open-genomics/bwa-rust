import { defineConfig } from 'vitepress'

// ============================================
// bwa-rust 技术文档站配置
// 定位为：BWA-MEM 算法的 Rust 实现文档
// 目标用户：生物信息学研究人员、Rust 开发者、算法学习者
// ============================================

export default defineConfig({
  // 基础配置
  title: 'bwa-rust',
  titleTemplate: ':title | bwa-rust',
  description: 'BWA-MEM 算法的 Rust 实现 —— 零 unsafe 代码，教育级可读性',
  lang: 'zh-CN',

  base: '/bwa-rust/',
  cleanUrls: true,
  lastUpdated: true,

  // SEO 和元数据
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/bwa-rust/logo.svg' }],
    ['meta', { name: 'theme-color', content: '#dea584' }],
    ['meta', { name: 'viewport', content: 'width=device-width, initial-scale=1.0' }],

    // Open Graph
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:site_name', content: 'bwa-rust' }],
    ['meta', { property: 'og:title', content: 'bwa-rust - BWA-MEM 算法的 Rust 实现' }],
    ['meta', { property: 'og:description', content: '零 unsafe 代码的 BWA-MEM 风格序列比对器实现' }],

    // Keywords
    ['meta', { name: 'keywords', content: 'bwa-rust,BWA,BWA-MEM,DNA,序列比对,生物信息学,Rust,FM索引' }],
  ],

  // Markdown 配置
  markdown: {
    theme: {
      light: 'github-light',
      dark: 'github-dark',
    },
    lineNumbers: true,
  },

  // 主题配置
  themeConfig: {
    logo: '/logo.svg',
    siteTitle: 'bwa-rust',

    // 导航栏 - 简洁清晰，全部中文
    nav: [
      { text: '首页', link: '/' },
      { text: '指南', link: '/guide/', activeMatch: '/guide/' },
      { text: '架构', link: '/architecture/', activeMatch: '/architecture/' },
      { text: '性能', link: '/benchmarks' },
      { text: 'API', link: 'https://docs.rs/bwa-rust' },
      {
        text: 'v0.2.0',
        items: [
          { text: '更新日志', link: '/changelog' },
          { text: '路线图', link: '/roadmap' },
          { text: '常见问题', link: '/faq' },
          { text: 'GitHub', link: 'https://github.com/LessUp/bwa-rust' },
        ],
      },
    ],

    // 侧边栏 - 仅配置实际存在的页面
    sidebar: {
      '/guide/': [
        {
          text: '使用指南',
          items: [
            { text: '概览', link: '/guide/' },
            { text: '安装', link: '/guide/installation' },
            { text: '快速开始', link: '/guide/quickstart' },
          ],
        },
      ],
      '/architecture/': [
        {
          text: '架构设计',
          items: [
            { text: '概览', link: '/architecture/' },
            { text: '核心算法', link: '/architecture/algorithms' },
            { text: '比对流水线', link: '/architecture/pipeline' },
          ],
        },
      ],
    },

    // 社交链接
    socialLinks: [
      { icon: 'github', link: 'https://github.com/LessUp/bwa-rust' },
    ],

    // 搜索
    search: {
      provider: 'local',
      options: {
        detailedView: true,
      },
    },

    // 页脚
    footer: {
      message: '基于 MIT 许可证发布',
      copyright: '© 2024-2026 LessUp',
    },

    // 编辑链接
    editLink: {
      pattern: 'https://github.com/LessUp/bwa-rust/edit/master/site/:path',
      text: '在 GitHub 上编辑此页',
    },

    // 最后更新时间
    lastUpdated: {
      text: '最后更新',
      formatOptions: {
        dateStyle: 'short',
      },
    },

    // 大纲
    outline: {
      label: '本页内容',
      level: [2, 3],
    },

    // 返回顶部
    returnToTopLabel: '返回顶部',

    // 侧边栏菜单
    sidebarMenuLabel: '菜单',

    // 暗黑模式
    darkModeSwitchLabel: '主题',
  },

  // Vite 配置
  vite: {
    css: {
      preprocessorOptions: {
        scss: {
          // 如需自定义 CSS 变量可在此配置
        },
      },
    },
  },
})
