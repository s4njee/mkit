import './style.css'
import './ui/ui.css'
import { cardBySlug } from './catalog'
import { mountDocs, renderDocs, renderNotFound } from './pages/docs'
import { mountHome, renderHome } from './pages/home'
import { mountTheming, renderTheming } from './pages/theming'
import { installLinkInterceptor, navigate, onPopState, resolveRoute } from './router'
import { mountThemeToggle } from './theme'

const app = document.querySelector<HTMLDivElement>('#app')!

let cleanup: (() => void) | undefined

function render() {
  cleanup?.()
  cleanup = undefined

  const route = resolveRoute(location.pathname)

  if (route.name === 'theming') {
    app.innerHTML = renderTheming()
    document.title = 'The mkit design language · mkit'
    mountTheming()
    mountThemeToggle()
    window.scrollTo({ top: 0, behavior: 'auto' })
    return
  }

  if (route.name === 'component') {
    const card = cardBySlug.get(route.slug)
    if (card) {
      app.innerHTML = renderDocs(card)
      document.title = `${card.name} · mkit`
      cleanup = mountDocs()
      mountThemeToggle()
      scrollToHash()
      return
    }
    app.innerHTML = renderNotFound(location.pathname)
    document.title = 'Not found · mkit'
    mountThemeToggle()
    return
  }

  if (route.name === 'not-found') {
    app.innerHTML = renderNotFound(location.pathname)
    document.title = 'Not found · mkit'
    mountThemeToggle()
    return
  }

  app.innerHTML = renderHome()
  document.title = 'mkit · Components for pro apps'
  cleanup = mountHome()

  scrollToHash()
}

function scrollToHash() {
  const target = location.hash ? document.getElementById(decodeURIComponent(location.hash.slice(1))) : null
  if (target) requestAnimationFrame(() => target.scrollIntoView())
  else window.scrollTo({ top: 0, behavior: 'auto' })
}

installLinkInterceptor((href) => {
  navigate(href)
  render()
})
onPopState(render)
render()
