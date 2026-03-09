import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'bwa-rust',
  description: 'A BWA-inspired sequence aligner implemented in Rust',
  base: '/bwa-rust/',

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/bwa-rust/logo.svg' }],
  ],

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
            text: '介绍',
            items: [
              { text: '快速开始', link: '/guide/getting-started' },
            ],
          },
          {
            text: '深入了解',
            items: [
              { text: '架构设计', link: '/guide/architecture' },
              { text: '算法教程', link: '/guide/tutorial' },
            ],
          },
          {
            text: '项目',
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
        docFooter: { prev: '上一页', next: '下一页' },
        outline: { label: '页面导航' },
        lastUpdated: { text: '最后更新于' },
        returnToTopLabel: '回到顶部',
        sidebarMenuLabel: '菜单',
        darkModeSwitchLabel: '主题',
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
            text: 'Introduction',
            items: [
              { text: 'Getting Started', link: '/en/guide/getting-started' },
            ],
          },
          {
            text: 'Deep Dive',
            items: [
              { text: 'Architecture', link: '/en/guide/architecture' },
              { text: 'Algorithm Tutorial', link: '/en/guide/tutorial' },
            ],
          },
          {
            text: 'Project',
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
      },
    },
  },

  themeConfig: {
    socialLinks: [
      { icon: 'github', link: 'https://github.com/LessUp/bwa-rust' },
    ],
    search: {
      provider: 'local',
    },
    externalLinkIcon: true,
  },
})
