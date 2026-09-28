// External links used across the site. The repository has no public remote
// yet, so these are the intended locations; update them in one place when the
// repository is published.
export const repoUrl = 'https://github.com/mk7s/mkit'
export const bookSourceUrl = `${repoUrl}/tree/main/book/src`
export const bookBlobUrl = `${repoUrl}/blob/main/book/src`
export const planUrl = `${repoUrl}/blob/main/plan.md`
export const registryBlobUrl = `${repoUrl}/blob/main`

const base = import.meta.env.BASE_URL || '/'
export const siteHref = (path = '') => `${base}${path.replace(/^\//, '')}`
