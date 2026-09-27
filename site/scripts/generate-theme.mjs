// Generate the token board data for the E9 theming page directly from
// crates/mkit-core/src/theme.rs, so the site cannot drift from the library.
//
// Run with `npm run content` (also runs before dev and build).

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const siteRoot = resolve(here, '..')
const repoRoot = resolve(siteRoot, '..')
const themeRs = join(repoRoot, 'crates', 'mkit-core', 'src', 'theme.rs')
const outFile = join(siteRoot, 'src', 'generated', 'theme.ts')

const source = readFileSync(themeRs, 'utf8')

function extractConsts(text) {
  const consts = {}
  const re = /(?:pub\s+)?const\s+([A-Z][A-Z0-9_]*)\s*:\s*[\w:]+(?:<[^>]*>)?\s*=\s*/g
  let match
  while ((match = re.exec(text))) {
    let depth = 0
    let end = -1
    for (let index = re.lastIndex; index < text.length; index += 1) {
      const char = text[index]
      if (char === '{' || char === '(' || char === '[') depth += 1
      else if (char === '}' || char === ')' || char === ']') depth -= 1
      else if (char === ';' && depth === 0) {
        end = index
        break
      }
    }
    if (end === -1) throw new Error(`unterminated const ${match[1]}`)
    consts[match[1]] = text.slice(re.lastIndex, end).trim()
    re.lastIndex = end + 1
  }
  return consts
}

function splitTopLevel(body) {
  const parts = []
  let depth = 0
  let start = 0
  for (let index = 0; index < body.length; index += 1) {
    const char = body[index]
    if (char === '{' || char === '(' || char === '[') depth += 1
    else if (char === '}' || char === ')' || char === ']') depth -= 1
    else if (char === ',' && depth === 0) {
      parts.push(body.slice(start, index))
      start = index + 1
    }
  }
  const tail = body.slice(start)
  if (tail.trim()) parts.push(tail)
  return parts.map((part) => part.trim()).filter(Boolean)
}

const consts = extractConsts(source)
const cache = {}

function parseConst(name) {
  if (name in cache) return cache[name]
  if (!(name in consts)) throw new Error(`unknown const ${name}`)
  const value = parseValue(consts[name])
  cache[name] = value
  return value
}

function hexToCss(hex) {
  return `#${(hex & 0xffffff).toString(16).padStart(6, '0')}`
}

function alphaToCss(hex) {
  return `rgba(${(hex >> 24) & 0xff}, ${(hex >> 16) & 0xff}, ${(hex >> 8) & 0xff}, ${((hex & 0xff) / 255).toFixed(3)})`
}

function parseCallArgs(inner) {
  return splitTopLevel(inner)
}

function parseValue(text) {
  const value = text.trim()
  if (/^[A-Z][A-Z0-9_]*$/.test(value)) return parseConst(value)
  const colorMatch = value.match(/^color\(0x([0-9a-fA-F]+)\)$/)
  if (colorMatch) return hexToCss(parseInt(colorMatch[1], 16))
  const rgbaMatch = value.match(/^rgba\(0x([0-9a-fA-F]+)\)$/)
  if (rgbaMatch) return alphaToCss(parseInt(rgbaMatch[1], 16))
  const shadowMatch = value.match(/^shadow\(([\s\S]*)\)$/)
  if (shadowMatch) {
    const [y, blur, color] = parseCallArgs(shadowMatch[1])
    return { y: Number(y), blur: Number(blur), color: parseValue(color) }
  }
  const structMatch = value.match(/^[A-Za-z_]\w*\s*\{([\s\S]*)\}$/)
  if (structMatch) {
    const object = {}
    for (const field of splitTopLevel(structMatch[1])) {
      const colon = field.indexOf(':')
      const key = field.slice(0, colon).trim()
      object[key] = parseValue(field.slice(colon + 1))
    }
    return object
  }
  if (/^-?\d+(\.\d+)?$/.test(value)) return Number(value)
  if (/^".*"$/.test(value)) return JSON.parse(value)
  throw new Error(`cannot parse value: ${value}`)
}

const themeConsts = {
  LIGHT: 'light',
  DARK: 'dark',
  HIGH_CONTRAST: 'high-contrast',
  SHADCN_LIGHT: 'shadcn-light',
  SHADCN_DARK: 'shadcn-dark',
}
const labels = {
  light: 'Light',
  dark: 'Dark',
  'high-contrast': 'High contrast',
  'shadcn-light': 'shadcn light',
  'shadcn-dark': 'shadcn dark',
}

const themes = {}
for (const [constName, key] of Object.entries(themeConsts)) {
  const theme = parseConst(constName)
  themes[key] = {
    key,
    label: labels[key],
    name: theme.name,
    colors: theme.colors,
    typography: theme.typography,
    spacing: theme.spacing,
    radii: theme.radii,
    borders: theme.borders,
    controls: theme.controls,
    shadows: theme.shadows,
    motion: theme.motion,
  }
}

const order = Object.values(themeConsts)
const banner = `// AUTO-GENERATED from crates/mkit-core/src/theme.rs — do not edit.\n// Run \`npm run content\` after changing the theme tokens.\n\n`
const body = `export type ShadowToken = { y: number; blur: number; color: string }

export type ThemeTokens = {
  key: string
  label: string
  name: string
  colors: Record<string, string>
  typography: Record<string, number>
  spacing: Record<string, number>
  radii: Record<string, number>
  borders: Record<string, number>
  controls: Record<string, number>
  shadows: Record<string, ShadowToken>
  motion: Record<string, number>
}

export const themeOrder: string[] = ${JSON.stringify(order, null, 2)}

export const themes: Record<string, ThemeTokens> = ${JSON.stringify(themes, null, 2)}
`
mkdirSync(dirname(outFile), { recursive: true })
writeFileSync(outFile, banner + body)
console.log(`theme: wrote ${order.length} built-in themes from theme.rs`)
