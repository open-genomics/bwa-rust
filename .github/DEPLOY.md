# Documentation Deployment

## Automatic Deployment

Documentation automatically deploys to GitHub Pages on push to the default branch (`master` in this repository):
- Workflow: `.github/workflows/pages.yml`
- Site: https://lessup.github.io/bwa-rust/

## Manual Build & Preview

```bash
# Development mode (hot reload)
npm run docs:dev

# Production build
npm run docs:build

# Preview production build
npm run docs:preview
```

## Configuration

Site configuration: `site/.vitepress/config.mts`

Key settings:
- `base: '/bwa-rust/'` - Must match repository name
- PWA configuration (manifest, icons, workbox)
- Search, SEO, and bilingual support

## Custom Domain

To use a custom domain:
1. Add `CNAME` file to `site/public/` with your domain
2. Configure DNS per [GitHub Pages docs](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site)
