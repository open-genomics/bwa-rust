import { defineConfig } from 'vitepress'

export default defineConfig({
  // ─────────────────────────────────────────────────────────────
  // Site Configuration
  // ─────────────────────────────────────────────────────────────
  title: 'bwa-rust',
  description: 'A high-performance BWA-MEM style DNA sequence aligner implemented in Rust',
  base: '/bwa-rust/',
  cleanUrls: true,
  lastUpdated: true,

  // ─────────────────────────────────────────────────────────────
  // Sitemap & SEO
  // ─────────────────────────────────────────────────────────────
  sitemap: {
    hostname: 'https://lessup.github.io/bwa-rust/',
    lastmodDateOnly: true,
  },

  head: [
    // Canonical URL
    ['link', { rel: 'canonical', href: 'https://lessup.github.io/bwa-rust/' }],

    // Favicon
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/bwa-rust/logo.svg' }],
    ['link', { rel: 'apple-touch-icon', sizes: '180x180', href: '/bwa-rust/apple-touch-icon.png' }],

    // Theme color
    ['meta', { name: 'theme-color', content: '#dea584' }],
    ['meta', { name: 'msapplication-TileColor', content: '#dea584' }],

    // Open Graph
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'bwa-rust' }],
    ['meta', { property: 'og:description', content: 'A high-performance BWA-MEM style DNA sequence aligner implemented in Rust' }],
    ['meta', { property: 'og:url', content: 'https://lessup.github.io/bwa-rust/' }],
    ['meta', { property: 'og:site_name', content: 'bwa-rust' }],
    ['meta', { property: 'og:locale', content: 'en_US' }],
    ['meta', { property: 'og:locale:alternate', content: 'zh_CN' }],

    // Twitter Card
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    ['meta', { name: 'twitter:title', content: 'bwa-rust' }],
    ['meta', { name: 'twitter:description', content: 'A BWA-inspired sequence aligner implemented in Rust' }],
    ['meta', { name: 'twitter:site', content: '@LessUp' }],

    // Keywords
    ['meta', { name: 'keywords', content: 'bioinformatics, sequence alignment, BWA, BWA-MEM, Rust, DNA, genomics, FM-index, Smith-Waterman' }],
  ],

  // ─────────────────────────────────────────────────────────────
  // Markdown Configuration
  // ─────────────────────────────────────────────────────────────
  markdown: {
    lineNumbers: false,
    math: false,
  },

  // ─────────────────────────────────────────────────────────────
  // Internationalization
  // ─────────────────────────────────────────────────────────────
  locales: {
    root: {
      label: '中文',
      lang: 'zh-CN',
      themeConfig: {
        nav: [
          { text: '指南', link: '/guide/getting-started' },
          { text: '架构', link: '/guide/architecture' },
          { text: '教程', link: '/guide/tutorial' },
          { text: '路线图', link: '/roadmap' },
        ],
        sidebar: [
          {
            text: '📖 介绍',
            collapsed: false,
            items: [
              { text: '快速开始', link: '/guide/getting-started' },
            ],
          },
          {
            text: '🔬 深入了解',
            collapsed: false,
            items: [
              { text: '架构设计', link: '/guide/architecture' },
              { text: '算法教程', link: '/guide/tutorial' },
            ],
          },
          {
            text: '📋 项目',
            collapsed: false,
            items: [
              { text: '路线图', link: '/roadmap' },
            ],
          },
        ],
        editLink: {
          pattern: 'https://github.com/LessUp/bwa-rust/edit/main/site/:path',
          text: '在 GitHub 上编辑此页',
        },
        footer: {
          message: '基于 MIT 许可证发布',
          copyright: '© 2026 LessUp',
        },
        docFooter: {
          prev: '上一页',
          next: '下一页',
        },
        outline: {
          label: '页面导航',
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
        lightModeSwitchTitle: '切换到浅色模式',
        darkModeSwitchTitle: '切换到深色模式',
      },
    },
    en: {
      label: 'English',
      lang: 'en-US',
      link: '/en/',
      themeConfig: {
        nav: [
          { text: 'Guide', link: '/en/guide/getting-started' },
          { text: 'Architecture', link: '/en/guide/architecture' },
          { text: 'Tutorial', link: '/en/guide/tutorial' },
          { text: 'Roadmap', link: '/en/roadmap' },
        ],
        sidebar: [
          {
            text: '📖 Introduction',
            collapsed: false,
            items: [
              { text: 'Getting Started', link: '/en/guide/getting-started' },
            ],
          },
          {
            text: '🔬 Deep Dive',
            collapsed: false,
            items: [
              { text: 'Architecture', link: '/en/guide/architecture' },
              { text: 'Algorithm Tutorial', link: '/en/guide/tutorial' },
            ],
          },
          {
            text: '📋 Project',
            collapsed: false,
            items: [
              { text: 'Roadmap', link: '/en/roadmap' },
            ],
          },
        ],
        editLink: {
          pattern: 'https://github.com/LessUp/bwa-rust/edit/main/site/:path',
          text: 'Edit this page on GitHub',
        },
        footer: {
          message: 'Released under the MIT License',
          copyright: '© 2026 LessUp',
        },
        docFooter: {
          prev: 'Previous',
          next: 'Next',
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

  // ─────────────────────────────────────────────────────────────
  // Theme Configuration
  // ─────────────────────────────────────────────────────────────
  themeConfig: {
    // Logo
    logo: '/logo.svg',

    // Site title for home page
    siteTitle: 'bwa-rust',

    // Social links
    socialLinks: [
      { icon: 'github', link: 'https://github.com/LessUp/bwa-rust', ariaLabel: 'GitHub' },
    ],

    // Search
    search: {
      provider: 'local',
      options: {
        detailedView: true,
        miniSearch: {
          searchOptions: {
            fuzzy: 0.2,
            prefix: true,
            boost: { title: 4, text: 2, titles: 1 },
          },
        },
      },
    },

    // External link icon
    externalLinkIcon: true,

    // Carbon Ads (optional)
    // carbonAds: {
    //   code: '...',
    //   placement: '...',
    // },
  },
})
