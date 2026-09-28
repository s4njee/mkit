export type Route =
  | { name: 'home' }
  | { name: 'theming' }
  | { name: 'health' }
  | { name: 'component'; slug: string }
  | { name: 'not-found' }

const base = (import.meta.env.BASE_URL || '/').replace(/\/$/, '')

export function resolveRoute(pathname: string): Route {
  let path = pathname
  // Strip every leading copy of the base so bookmarked URLs from the
  // double-prefix regression still resolve instead of 404ing.
  if (base) {
    while (path.startsWith(base) && (path.length === base.length || path[base.length] === '/')) {
      path = path.slice(base.length) || '/'
    }
  }
  if (!path.startsWith('/')) path = `/${path}`
  path = path.replace(/\/+$/, '') || '/'
  if (path === '/') return { name: 'home' }
  if (path === '/components') return { name: 'home' }
  if (path === '/theming') return { name: 'theming' }
  if (path === '/health') return { name: 'health' }
  const match = path.match(/^\/components\/([A-Za-z0-9-]+)$/)
  if (match) return { name: 'component', slug: match[1] }
  return { name: 'not-found' }
}

export function hrefFor(path: string) {
  // Rendered hrefs already carry the base; never prefix twice.
  if (!base || path === base || path.startsWith(`${base}/`)) return path
  return `${base}${path}`
}

export function navigate(path: string, options: { replace?: boolean } = {}) {
  // The link interceptor forwards full hrefs (base included), so only
  // prefix bare app paths. Otherwise every click doubles the base.
  const url = !base || path === base || path.startsWith(`${base}/`) ? path : `${base}${path}`
  if (options.replace) history.replaceState(null, '', url)
  else history.pushState(null, '', url)
}

export function installLinkInterceptor(onNavigate: (path: string) => void) {
  document.addEventListener('click', (event) => {
    if (
      event.defaultPrevented ||
      event.button !== 0 ||
      event.metaKey ||
      event.ctrlKey ||
      event.shiftKey ||
      event.altKey
    ) {
      return
    }
    const target = event.target as HTMLElement | null
    const anchor = target?.closest?.('a')
    if (!anchor) return
    const href = anchor.getAttribute('href')
    if (!href) return
    if (anchor.getAttribute('target') === '_blank' || anchor.hasAttribute('download')) return
    // Static files (llms.txt) and explicit opt-outs load natively.
    if (anchor.hasAttribute('data-native') || /\.[a-z0-9]+$/i.test(href.split(/[?#]/)[0])) return
    if (/^(https?:|mailto:|tel:)/i.test(href)) return
    // Same-page hash links should scroll natively.
    if (href.startsWith('#')) return
    event.preventDefault()
    onNavigate(href)
  })
}

export function onPopState(callback: () => void) {
  window.addEventListener('popstate', callback)
}
