// External links used across the site. Update the repository in one place.
export const repoUrl = 'https://github.com/s4njee/mkit'
export const bookSourceUrl = `${repoUrl}/tree/main/book/src`
export const bookBlobUrl = `${repoUrl}/blob/main/book/src`
export const planUrl = `${repoUrl}/blob/main/plan.md`
export const registryBlobUrl = `${repoUrl}/blob/main`

const base = import.meta.env.BASE_URL || '/'
export const siteHref = (path = '') => `${base}${path.replace(/^\//, '')}`
