import { icon } from './icons'

const storageKey = 'mkit-site-theme'

function readSaved() {
  try {
    return localStorage.getItem(storageKey) === 'light'
  } catch {
    return false
  }
}

let lightMode = readSaved()
document.body.classList.toggle('light-mode', lightMode)

export const isLightMode = () => lightMode

function paintToggles() {
  document.querySelectorAll<HTMLButtonElement>('.theme-toggle').forEach((toggle) => {
    toggle.innerHTML = icon(lightMode ? 'moon' : 'sun')
    toggle.setAttribute('aria-label', lightMode ? 'Switch to dark mode' : 'Switch to light mode')
  })
}

export function mountThemeToggle() {
  paintToggles()
  document.querySelectorAll<HTMLButtonElement>('.theme-toggle').forEach((button) =>
    button.addEventListener('click', () => {
      lightMode = !lightMode
      document.body.classList.toggle('light-mode', lightMode)
      try {
        localStorage.setItem(storageKey, lightMode ? 'light' : 'dark')
      } catch {
        // Storage can be unavailable; the toggle still works for this visit.
      }
      paintToggles()
      // Component previews follow the site theme until toggled individually.
      const mode = lightMode ? 'light' : 'dark'
      document.querySelectorAll<HTMLElement>('.preview-canvas').forEach((canvas) => (canvas.dataset.uiTheme = mode))
      document.querySelectorAll<HTMLButtonElement>('[data-preview-theme]').forEach((toggle) => {
        toggle.innerHTML = icon(lightMode ? 'moon' : 'sun', 14)
        toggle.setAttribute('aria-label', `Switch preview to ${lightMode ? 'dark' : 'light'} theme`)
      })
    }),
  )
}
