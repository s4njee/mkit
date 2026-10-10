// Generate the token board data for the E9 theming page directly from
// crates/mkit-core/src/theme.rs and subcompact.rs, so the site cannot drift
// from the library.
//
// Run with `npm run content` (also runs before dev and build).

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const siteRoot = resolve(here, '..')
const repoRoot = resolve(siteRoot, '..')
const coreSrc = join(repoRoot, 'crates', 'mkit-core', 'src')
const outFile = join(siteRoot, 'src', 'generated', 'theme.ts')

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

// Each source file gets its own constant table, because both files define
// scale constants such as `TYPE` and `SPACE`.
const sources = {
  theme: extractConsts(readFileSync(join(coreSrc, 'theme.rs'), 'utf8')),
  subcompact: extractConsts(readFileSync(join(coreSrc, 'subcompact.rs'), 'utf8')),
}
const caches = { theme: {}, subcompact: {} }
let currentSource = 'theme'

function parseConst(name, source = currentSource) {
  const cache = caches[source]
  if (name in cache) return cache[name]
  const consts = sources[source]
  if (!(name in consts)) throw new Error(`unknown const ${name} in ${source}.rs`)
  const previous = currentSource
  currentSource = source
  try {
    cache[name] = parseValue(consts[name])
  } finally {
    currentSource = previous
  }
  return cache[name]
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

// [source file, const name, key]
const themeConsts = [
  ['theme', 'LIGHT', 'light'],
  ['theme', 'DARK', 'dark'],
  ['theme', 'HIGH_CONTRAST', 'high-contrast'],
  ['theme', 'SHADCN_LIGHT', 'shadcn-light'],
  ['theme', 'SHADCN_DARK', 'shadcn-dark'],
  ['subcompact', 'PHOSPHOR', 'subcompact-phosphor'],
  ['subcompact', 'AMBER', 'subcompact-amber'],
  ['subcompact', 'PAPER', 'subcompact-paper'],
]
const labels = {
  light: 'Light',
  dark: 'Dark',
  'high-contrast': 'High contrast',
  'shadcn-light': 'shadcn light',
  'shadcn-dark': 'shadcn dark',
  'subcompact-phosphor': 'Subcompact phosphor',
  'subcompact-amber': 'Subcompact amber',
  'subcompact-paper': 'Subcompact paper',
}

const themes = {}
for (const [source, constName, key] of themeConsts) {
  const theme = parseConst(constName, source)
  if (theme.name !== key) throw new Error(`${source}.rs ${constName} is named ${theme.name}, expected ${key}`)
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

const order = themeConsts.map(([, , key]) => key)
const banner = `// AUTO-GENERATED from crates/mkit-core/src/theme.rs and subcompact.rs — do not edit.\n// Run \`npm run content\` after changing the theme tokens.\n\n`
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
console.log(`theme: wrote ${order.length} built-in themes from theme.rs and subcompact.rs`)
