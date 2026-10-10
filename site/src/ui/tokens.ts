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
    // shadcn keeps destructive text light in both modes; the light theme's
    // background and dark theme's text are those light values.
    'destructive-foreground': mode === 'light' ? c.background : c.text,
    overlay: `color-mix(in srgb, ${mode === 'light' ? c.text : c.background} 50%, transparent)`,
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

/** The subcompact palettes, keyed as `data-ui-theme` values. */
export const subcompactPalettes = ['subcompact-phosphor', 'subcompact-amber', 'subcompact-paper'] as const
export type SubcompactPalette = (typeof subcompactPalettes)[number]

// A subcompact palette maps onto the same --ui-* names, plus --sc-* density
// variables for src/ui/subcompact.css. Unlike the shadcn mapping, borders and
// inputs use the palette's own `border` token, which meets 3:1 on every
// surface, and every shadow is the palette's (transparent) shadow.
function subcompactRule(key: SubcompactPalette) {
  const theme = themes[key]
  const c = theme.colors
  const dark = key !== 'subcompact-paper'
  const mix = dark ? 12 : 4
  const vars: Record<string, string> = {
    '--ui-background': c.background,
    '--ui-foreground': c.text,
    '--ui-card': c.surface,
    '--ui-popover': c.elevated_surface,
    '--ui-primary': c.accent,
    '--ui-primary-foreground': c.accent_text,
    '--ui-muted-foreground': c.text_muted,
    '--ui-border': c.border,
    '--ui-input': c.border,
    '--ui-ring': c.focus,
    '--ui-destructive': c.danger,
    '--ui-destructive-foreground': c.accent_text,
    '--ui-overlay': `color-mix(in srgb, ${c.background} 70%, transparent)`,
    '--ui-success': c.success,
    '--ui-warning': c.warning,
    '--ui-muted': `color-mix(in srgb, ${c.text} ${mix}%, ${c.background})`,
    '--ui-accent': `color-mix(in srgb, ${c.text} ${mix}%, ${c.background})`,
    '--ui-secondary': `color-mix(in srgb, ${c.text} ${mix}%, ${c.background})`,
    '--ui-radius-sm': `${theme.radii.small}px`,
    '--ui-radius-md': `${theme.radii.medium}px`,
    '--ui-radius-lg': `${theme.radii.large}px`,
    '--ui-shadow-xs': shadow(theme.shadows.small),
    '--ui-shadow-sm': shadow(theme.shadows.small),
    '--ui-shadow-md': shadow(theme.shadows.medium),
    '--ui-shadow-lg': shadow(theme.shadows.large),
    '--sc-surface': c.surface,
    '--sc-elevated': c.elevated_surface,
    '--sc-accent': c.accent,
    '--sc-success': c.success,
    '--sc-warning': c.warning,
    '--sc-danger': c.danger,
    '--sc-hairline': `${theme.borders.hairline}px`,
    '--sc-motion-fast': `${theme.motion.fast_ms}ms`,
  }
  for (const [token, value] of Object.entries(theme.controls)) vars[`--sc-control-${token}`] = `${value}px`
  for (const [token, value] of Object.entries(theme.spacing)) vars[`--sc-space-${token}`] = `${value}px`
  for (const [token, value] of Object.entries(theme.typography)) {
    vars[`--sc-type-${token.replace(/_/g, '-')}`] = `${value}px`
  }
  const body = Object.entries(vars)
    .map(([name, value]) => `${name}: ${value};`)
    .join(' ')
  return `.ui[data-ui-theme="${key}"], [data-ui-theme="${key}"] .ui { ${body} color-scheme: ${dark ? 'dark' : 'light'}; }`
}

let installed = false

/** Inject the --ui-* variables once. Safe to call on every render. */
export function installUiTokens() {
  if (installed) return
  installed = true
  const style = document.createElement('style')
  style.dataset.source = 'mkit-core shadcn and subcompact themes'
  style.textContent = [
    rule('light', themes['shadcn-light']),
    rule('dark', themes['shadcn-dark']),
    ...subcompactPalettes.map(subcompactRule),
  ].join('\n')
  document.head.append(style)
}
