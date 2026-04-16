import { defineConfig } from 'vitepress'
import { withPwa } from '@vite-pwa/vitepress'

// ============================================
// 🚀 BWA-RUST: Ultra-Modern VitePress Config
// ============================================

const baseConfig = defineConfig({
  // Core Configuration
  title: 'bwa-rust',
  titleTemplate: ':title | bwa-rust - DNA Sequence Aligner',
  description: 'A high-performance BWA-MEM style DNA sequence aligner implemented in Rust with zero unsafe code',
  lang: 'en-US',
  
  base: '/bwa-rust/',
  cleanUrls: true,
  lastUpdated: true,
  
  ignoreDeadLinks: [
    /^https?:\/\/localhost/,
    /^https?:\/\/127\.0\.0\.1/,
    /github\.com/,
  ],
  
  // Head - SEO + Performance
  head: [
    ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/bwa-rust/logo.svg' }],
    ['link', { rel: 'apple-touch-icon', sizes: '180x180', href: '/bwa-rust/apple-touch-icon.png' }],
    ['link', { rel: 'mask-icon', href: '/bwa-rust/logo.svg', color: '#646cff' }],
    
    ['meta', { name: 'theme-color', media: '(prefers-color-scheme: light)', content: '#ffffff' }],
    ['meta', { name: 'theme-color', media: '(prefers-color-scheme: dark)', content: '#1a1a1a' }],
    ['meta', { name: 'msapplication-TileColor', content: '#646cff' }],
    ['meta', { name: 'viewport', content: 'width=device-width, initial-scale=1.0, viewport-fit=cover' }],
    
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:site_name', content: 'bwa-rust' }],
    ['meta', { property: 'og:locale', content: 'en_US' }],
    ['meta', { property: 'og:locale:alternate', content: 'zh_CN' }],
    
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    
    ['meta', { name: 'keywords', content: 'bwa-rust,BWA,BWA-MEM,sequence alignment,DNA,bioinformatics,Rust,FM-index' }],
    ['meta', { name: 'robots', content: 'index, follow' }],
  ],
  
  // Markdown
  markdown: {
    theme: {
      light: 'github-light',
      dark: 'github-dark',
    },
    lineNumbers: true,
  },
  
  // Vite Configuration
  vite: {
    ssr: {
      noExternal: ['@vite-pwa/vitepress'],
    },
    build: {
      chunkSizeWarningLimit: 1600,
    },
  },
  
  // Locales
  locales: {
    root: {
      label: '简体中文',
      lang: 'zh-CN',
      description: '受 BWA/BWA-MEM 启发的高性能 Rust DNA 序列比对器',
      link: '/',
      themeConfig: {
        nav: [
          { text: '📖 文档', link: '/guide/getting-started', activeMatch: '/guide/' },
          { 
            text: '🔬 架构',
            items: [
              { text: '架构概述', link: '/guide/architecture' },
              { text: '索引构建', link: '/guide/index-building' },
              { text: '比对算法', link: '/guide/alignment' },
              { text: '性能优化', link: '/guide/performance' },
            ],
          },
          { text: '📚 API', link: '/api/', activeMatch: '/api/' },
          { text: '🗺️ 路线图', link: '/roadmap' },
          {
            text: '📦 v0.2.0',
            items: [
              { text: '更新日志', link: '/changelog' },
              { text: 'GitHub Releases', link: 'https://github.com/LessUp/bwa-rust/releases' },
            ],
          },
        ],
        
        sidebar: {
          '/guide/': {
            base: '/guide/',
            items: [
              {
                text: '🚀 开始',
                collapsed: false,
                items: [
                  { text: '快速入门', link: 'getting-started' },
                  { text: '安装指南', link: 'installation' },
                  { text: '第一个示例', link: 'first-example' },
                ],
              },
              {
                text: '🔬 核心概念',
                collapsed: false,
                items: [
                  { text: '架构总览', link: 'architecture' },
                  { text: '索引构建', link: 'index-building' },
                  { text: '比对流程', link: 'alignment' },
                  { text: '性能特性', link: 'performance' },
                ],
              },
              {
                text: '🧬 算法详解',
                collapsed: false,
                items: [
                  { text: 'FM 索引原理', link: 'fm-index' },
                  { text: 'SMEM 种子', link: 'smem-seeds' },
                  { text: 'Smith-Waterman', link: 'smith-waterman' },
                  { text: '种子链', link: 'seed-chains' },
                ],
              },
            ],
          },
          '/api/': {
            base: '/api/',
            items: [
              {
                text: '📚 API 参考',
                collapsed: false,
                items: [
                  { text: 'API 概览', link: '' },
                  { text: 'Index 模块', link: 'index' },
                  { text: 'Align 模块', link: 'align' },
                  { text: 'IO 模块', link: 'io' },
                  { text: 'Util 模块', link: 'util' },
                ],
              },
            ],
          },
        },
        
        editLink: {
          pattern: 'https://github.com/LessUp/bwa-rust/edit/main/site/:path',
          text: '在 GitHub 上编辑此页',
        },
        
        footer: {
          message: '基于 MIT 许可证发布',
          copyright: '© 2024-2026 LessUp',
        },
        
        docFooter: {
          prev: '上一页',
          next: '下一页',
        },
        
        outline: {
          label: '本页目录',
          level: [2, 3],
        },
        
        lastUpdated: {
          text: '最后更新于',
          formatOptions: {
            dateStyle: 'medium',
            timeStyle: 'short',
          },
        },
        
        returnToTopLabel: '回到顶部',
        sidebarMenuLabel: '菜单',
        darkModeSwitchLabel: '主题',
      },
    },
    en: {
      label: 'English',
      lang: 'en-US',
      description: 'A high-performance BWA-MEM style DNA sequence aligner in Rust',
      link: '/en/',
      themeConfig: {
        nav: [
          { text: '📖 Docs', link: '/en/guide/getting-started', activeMatch: '/en/guide/' },
          { 
            text: '🔬 Architecture',
            items: [
              { text: 'Overview', link: '/en/guide/architecture' },
              { text: 'Index Building', link: '/en/guide/index-building' },
              { text: 'Alignment', link: '/en/guide/alignment' },
              { text: 'Performance', link: '/en/guide/performance' },
            ],
          },
          { text: '📚 API', link: '/en/api/', activeMatch: '/en/api/' },
          { text: '🗺️ Roadmap', link: '/en/roadmap' },
          {
            text: '📦 v0.2.0',
            items: [
              { text: 'Changelog', link: '/en/changelog' },
              { text: 'GitHub Releases', link: 'https://github.com/LessUp/bwa-rust/releases' },
            ],
          },
        ],
        
        sidebar: {
          '/en/guide/': {
            base: '/en/guide/',
            items: [
              {
                text: '🚀 Getting Started',
                collapsed: false,
                items: [
                  { text: 'Quick Start', link: 'getting-started' },
                  { text: 'Installation', link: 'installation' },
                  { text: 'First Example', link: 'first-example' },
                ],
              },
              {
                text: '🔬 Core Concepts',
                collapsed: false,
                items: [
                  { text: 'Overview', link: 'architecture' },
                  { text: 'Index Building', link: 'index-building' },
                  { text: 'Alignment', link: 'alignment' },
                  { text: 'Performance', link: 'performance' },
                ],
              },
              {
                text: '🧬 Algorithms',
                collapsed: false,
                items: [
                  { text: 'FM Index', link: 'fm-index' },
                  { text: 'SMEM Seeds', link: 'smem-seeds' },
                  { text: 'Smith-Waterman', link: 'smith-waterman' },
                  { text: 'Seed Chains', link: 'seed-chains' },
                ],
              },
            ],
          },
          '/en/api/': {
            base: '/en/api/',
            items: [
              {
                text: '📚 API Reference',
                collapsed: false,
                items: [
                  { text: 'Overview', link: '' },
                  { text: 'Index', link: 'index' },
                  { text: 'Align', link: 'align' },
                  { text: 'IO', link: 'io' },
                  { text: 'Util', link: 'util' },
                ],
              },
            ],
          },
        },
        
        editLink: {
          pattern: 'https://github.com/LessUp/bwa-rust/edit/main/site/:path',
          text: 'Edit this page on GitHub',
        },
        
        footer: {
          message: 'Released under the MIT License',
          copyright: '© 2024-2026 LessUp',
        },
        
        outline: {
          label: 'On this page',
          level: [2, 3],
        },
        
        lastUpdated: {
          text: 'Last updated',
          formatOptions: {
            dateStyle: 'medium',
            timeStyle: 'short',
          },
        },
      },
    },
  },
  
  // Theme Config
  themeConfig: {
    logo: '/logo.svg',
    siteTitle: 'bwa-rust',
    
    socialLinks: [
      { icon: 'github', link: 'https://github.com/LessUp/bwa-rust' },
    ],
    
    search: {
      provider: 'local',
      options: {
        detailedView: true,
        miniSearch: {
          options: {
            fields: ['title', 'titles', 'text'],
            searchOptions: {
              fuzzy: 0.2,
              prefix: true,
              boost: { title: 5, titles: 3, text: 1 },
            },
          },
        },
      },
    },
    
    externalLinkIcon: true,
  },
})

// PWA Configuration
const pwaConfig = {
  mode: 'production',
  strategies: 'generateSW',
  registerType: 'autoUpdate',
  injectRegister: 'auto',
  
  manifest: {
    name: 'bwa-rust Documentation',
    short_name: 'bwa-rust',
    description: 'A high-performance BWA-MEM style DNA sequence aligner in Rust',
    theme_color: '#646cff',
    background_color: '#ffffff',
    display: 'standalone',
    scope: '/bwa-rust/',
    start_url: '/bwa-rust/',
    icons: [
      { src: '/bwa-rust/icons/icon-192x192.png', sizes: '192x192', type: 'image/png' },
      { src: '/bwa-rust/icons/icon-512x512.png', sizes: '512x512', type: 'image/png' },
    ],
  },
  
  workbox: {
    globPatterns: ['**/*.{js,css,html,svg,png,ico,txt,woff2}'],
    cleanupOutdatedCaches: true,
  },
  
  devOptions: {
    enabled: false,
  },
}

// Export with PWA
export default withPwa(baseConfig, pwaConfig)
