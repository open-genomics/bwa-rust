import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'bwa-rust',
  titleTemplate: ':title | bwa-rust',
  description: '内存安全的 BWA-MEM 风格单端 DNA 短读比对器',
  lang: 'zh-CN',
  base: '/bwa-rust/',
  cleanUrls: true,
  lastUpdated: true,

  head: [
    ['meta', { name: 'theme-color', content: '#6d4aff' }],
    ['meta', { name: 'viewport', content: 'width=device-width, initial-scale=1.0' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:site_name', content: 'bwa-rust' }],
    ['meta', { property: 'og:title', content: 'bwa-rust - Rust BWA-MEM style aligner' }],
    ['meta', { property: 'og:description', content: 'FM-index、SMEM、链构建、Smith-Waterman 和 SAM 输出的 Rust 实现。' }],
    ['meta', { name: 'keywords', content: 'bwa-rust,BWA-MEM,DNA,sequence alignment,bioinformatics,Rust,FM-index,SMEM' }],
  ],

  markdown: {
    theme: {
      light: 'github-light',
      dark: 'github-dark',
    },
    lineNumbers: true,
  },

  themeConfig: {
    siteTitle: 'bwa-rust',
    nav: [
      { text: '首页', link: '/' },
      { text: '指南', link: '/guide/', activeMatch: '/guide/' },
      { text: '架构', link: '/architecture/', activeMatch: '/architecture/' },
      { text: '性能', link: '/benchmarks' },
      { text: 'FAQ', link: '/faq' },
      { text: 'API', link: 'https://docs.rs/bwa-rust' },
      { text: 'GitHub', link: 'https://github.com/LessUp/bwa-rust' },
    ],
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
          text: '架构',
          items: [
            { text: '概览', link: '/architecture/' },
            { text: '核心算法', link: '/architecture/algorithms' },
            { text: '比对流水线', link: '/architecture/pipeline' },
          ],
        },
      ],
    },
    socialLinks: [{ icon: 'github', link: 'https://github.com/LessUp/bwa-rust' }],
    search: { provider: 'local' },
    footer: {
      message: 'MIT License. Inspired by BWA.',
      copyright: 'Copyright © 2024-2026 LessUp',
    },
    editLink: {
      pattern: 'https://github.com/LessUp/bwa-rust/edit/master/site/:path',
      text: '在 GitHub 上编辑此页',
    },
    lastUpdated: {
      text: '最后更新',
      formatOptions: { dateStyle: 'short' },
    },
    outline: { label: '本页内容', level: [2, 3] },
    returnToTopLabel: '返回顶部',
    sidebarMenuLabel: '菜单',
    darkModeSwitchLabel: '主题',
  },
})
