import { themes } from '../generated/theme'
import type { ShadowToken, ThemeTokens } from '../generated/theme'

// Map mkit-core's shadcn themes onto shadcn's variable names. Muted, accent,
// and secondary are not separate mkit roles, so they are mixed from text and
// background the same way the GPUI components derive hover fills.
const shadow = (value: ShadowToken | undefined) => (value ? `0 ${value.y}px ${value.blur}px ${value.color}` : 'none')

function rule(mode: 'light' | 'dark', theme: ThemeTokens) {
  const c = theme.colors
  const mix = mode === 'light' ? 4 : 12
  const vars: Record<string, string> = {
    background: c.background,
    foreground: c.text,
    card: c.surface,
    popover: c.surface,
    primary: c.accent,
    'primary-foreground': c.accent_text,
    'muted-foreground': c.text_muted,
    border: mode === 'light' ? c.border : `color-mix(in srgb, ${c.text} 10%, transparent)`,
    input: mode === 'light' ? c.border : `color-mix(in srgb, ${c.text} 15%, transparent)`,
    ring: c.focus,
    destructive: c.danger,
    success: c.success,
    warning: c.warning,
    muted: `color-mix(in srgb, ${c.text} ${mix}%, ${c.background})`,
    accent: `color-mix(in srgb, ${c.text} ${mix}%, ${c.background})`,
    secondary: `color-mix(in srgb, ${c.text} ${mix}%, ${c.background})`,
    'radius-sm': `${theme.radii.small}px`,
    'radius-md': `${theme.radii.medium}px`,
    'radius-lg': `${theme.radii.large}px`,
    'shadow-xs': shadow(theme.shadows.small),
    'shadow-sm': shadow(theme.shadows.small),
    'shadow-md': shadow(theme.shadows.medium),
    'shadow-lg': shadow(theme.shadows.large),
  }
  const body = Object.entries(vars)
    .map(([key, value]) => `--ui-${key}: ${value};`)
    .join(' ')
  return `.ui[data-ui-theme="${mode}"], [data-ui-theme="${mode}"] .ui { ${body} color-scheme: ${mode}; }`
}

let installed = false

/** Inject the --ui-* variables once. Safe to call on every render. */
export function installUiTokens() {
  if (installed) return
  installed = true
  const style = document.createElement('style')
  style.dataset.source = 'mkit-core shadcn themes'
  style.textContent = [rule('light', themes['shadcn-light']), rule('dark', themes['shadcn-dark'])].join('\n')
  document.head.append(style)
}
