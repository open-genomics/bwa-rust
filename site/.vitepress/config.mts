import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'
import llmstxt from 'vitepress-plugin-llms'

const rawBase = process.env.VITEPRESS_BASE
const base = rawBase
  ? rawBase.startsWith('/')
    ? rawBase.endsWith('/') ? rawBase : `${rawBase}/`
    : `/${rawBase}/`
  : '/bwa-rust/'

export default withMermaid(defineConfig({
  base,
  title: 'bwa-rust',
  cleanUrls: true,
  lastUpdated: true,

  sitemap: {
    hostname: 'https://lessup.github.io/bwa-rust/'
  },

  head: [
    ['meta', { name: 'theme-color', content: '#6d4aff' }],
    ['meta', { name: 'viewport', content: 'width=device-width, initial-scale=1.0' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:site_name', content: 'bwa-rust' }],
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
  ],

  markdown: {
    theme: {
      light: 'github-light',
      dark: 'github-dark',
    },
    lineNumbers: true,
  },

  locales: {
    zh: {
      label: '简体中文',
      lang: 'zh-CN',
      link: '/zh/',
      title: 'bwa-rust',
      description: '内存安全的 BWA-MEM 风格单端 DNA 短读比对器',
      themeConfig: {
        nav: [
          { text: '首页', link: '/zh/' },
          { text: '架构', link: '/zh/architecture/', activeMatch: '/zh/architecture/' },
          { text: '深度', link: '/zh/deep-dive/', activeMatch: '/zh/deep-dive/' },
          { text: '指南', link: '/zh/guide/', activeMatch: '/zh/guide/' },
          { text: '规范', link: '/zh/specs/', activeMatch: '/zh/specs/' },
          { text: 'API', link: 'https://docs.rs/bwa-rust' },
          { text: 'GitHub', link: 'https://github.com/LessUp/bwa-rust' },
        ],
        sidebar: {
          '/zh/guide/': [
            {
              text: '使用指南',
              items: [
                { text: '概览', link: '/zh/guide/' },
                { text: '安装', link: '/zh/guide/installation' },
                { text: '快速开始', link: '/zh/guide/quickstart' },
              ],
            },
          ],
          '/zh/architecture/': [
            {
              text: '技术架构',
              items: [
                { text: '架构概览', link: '/zh/architecture/' },
                { text: '核心算法', link: '/zh/architecture/algorithms' },
                { text: '比对流水线', link: '/zh/architecture/pipeline' },
              ],
            },
          ],
          '/zh/deep-dive/': [
            {
              text: '技术深度',
              items: [
                { text: '索引格式', link: '/zh/deep-dive/' },
                { text: '内存安全', link: '/zh/deep-dive/memory-safety' },
                { text: '性能分析', link: '/zh/deep-dive/performance' },
              ],
            },
          ],
          '/zh/specs/': [
            {
              text: '规范与验证',
              items: [
                { text: 'OpenSpec 规范', link: '/zh/specs/' },
                { text: '验证边界', link: '/zh/specs/validation' },
                { text: '限制声明', link: '/zh/specs/limitations' },
              ],
            },
          ],
        },
        editLink: {
          pattern: 'https://github.com/LessUp/bwa-rust/edit/master/site/:path',
          text: '在 GitHub 上编辑此页',
        },
        outline: { label: '本页内容', level: [2, 3] },
        lastUpdated: {
          text: '最后更新',
          formatOptions: { dateStyle: 'short' },
        },
        returnToTopLabel: '返回顶部',
        sidebarMenuLabel: '菜单',
        darkModeSwitchLabel: '主题',
        docFooter: {
          prev: '上一页',
          next: '下一页',
        },
      },
    },
    en: {
      label: 'English',
      lang: 'en-US',
      link: '/en/',
      title: 'bwa-rust',
      description: 'Memory-safe BWA-MEM style single-end DNA short-read aligner',
      themeConfig: {
        nav: [
          { text: 'Home', link: '/en/' },
          { text: 'Architecture', link: '/en/architecture/', activeMatch: '/en/architecture/' },
          { text: 'Deep Dive', link: '/en/deep-dive/', activeMatch: '/en/deep-dive/' },
          { text: 'Guide', link: '/en/guide/', activeMatch: '/en/guide/' },
          { text: 'Specs', link: '/en/specs/', activeMatch: '/en/specs/' },
          { text: 'API', link: 'https://docs.rs/bwa-rust' },
          { text: 'GitHub', link: 'https://github.com/LessUp/bwa-rust' },
        ],
        sidebar: {
          '/en/guide/': [
            {
              text: 'Guide',
              items: [
                { text: 'Overview', link: '/en/guide/' },
                { text: 'Installation', link: '/en/guide/installation' },
                { text: 'Quick Start', link: '/en/guide/quickstart' },
              ],
            },
          ],
          '/en/architecture/': [
            {
              text: 'Architecture',
              items: [
                { text: 'Overview', link: '/en/architecture/' },
                { text: 'Core Algorithms', link: '/en/architecture/algorithms' },
                { text: 'Alignment Pipeline', link: '/en/architecture/pipeline' },
              ],
            },
          ],
          '/en/deep-dive/': [
            {
              text: 'Deep Dive',
              items: [
                { text: 'Index Format', link: '/en/deep-dive/' },
                { text: 'Memory Safety', link: '/en/deep-dive/memory-safety' },
                { text: 'Performance', link: '/en/deep-dive/performance' },
              ],
            },
          ],
          '/en/specs/': [
            {
              text: 'Specs',
              items: [
                { text: 'OpenSpec', link: '/en/specs/' },
                { text: 'Validation', link: '/en/specs/validation' },
                { text: 'Limitations', link: '/en/specs/limitations' },
              ],
            },
          ],
        },
        editLink: {
          pattern: 'https://github.com/LessUp/bwa-rust/edit/master/site/:path',
          text: 'Edit this page on GitHub',
        },
        outline: { label: 'On this page', level: [2, 3] },
        lastUpdated: {
          text: 'Last updated',
          formatOptions: { dateStyle: 'short' },
        },
        returnToTopLabel: 'Return to top',
        sidebarMenuLabel: 'Menu',
        darkModeSwitchLabel: 'Theme',
        docFooter: {
          prev: 'Previous page',
          next: 'Next page',
        },
      },
    },
  },

  themeConfig: {
    socialLinks: [{ icon: 'github', link: 'https://github.com/LessUp/bwa-rust' }],
    search: { provider: 'local' },
    footer: {
      message: 'MIT License. Inspired by BWA.',
      copyright: 'Copyright © 2024-2026 LessUp',
    },
  },

  vite: {
    plugins: [llmstxt()],
  },
}))
