// Build-time content pipeline for the mkit component site.
//
// The book in ../book is the single source of truth for component documentation.
// This script reads the component chapters, resolves mdBook {{#include}} anchors,
// rewrites relative links and images, renders Markdown to HTML, copies the
// referenced screenshots into public/, and writes src/generated/components.ts.
//
// Run it with `npm run content` (also runs automatically before dev and build).

import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { dirname, join, relative, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'
import { Marked } from 'marked'
import hljs from 'highlight.js/lib/core'
import rust from 'highlight.js/lib/languages/rust'
import bash from 'highlight.js/lib/languages/bash'

hljs.registerLanguage('rust', rust)
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('sh', bash)

const here = dirname(fileURLToPath(import.meta.url))
const siteRoot = resolve(here, '..')
const repoRoot = resolve(siteRoot, '..')
const bookSrc = join(repoRoot, 'book', 'src')
const bookComponents = join(bookSrc, 'components')
const catalogPath = join(siteRoot, 'content', 'catalog.json')
const publicBook = join(siteRoot, 'public', 'book')

// Harness screenshots are captured at 2x unless their name says -1x. Give each
// book image its CSS-pixel size so a 2x capture displays at the size the
// component actually renders, matching the web previews beside it.
function bookImageSize(href) {
  const match = /^@book\/(.+)$/.exec(href || '')
  if (!match || !match[1].endsWith('.png')) return undefined
  const file = join(bookSrc, match[1])
  if (!existsSync(file)) return undefined
  const header = readFileSync(file).subarray(16, 24)
  const scale = /-1x\.png$/.test(match[1]) ? 1 : 2
  return {
    width: Math.round(header.readUInt32BE(0) / scale),
    height: Math.round(header.readUInt32BE(4) / scale),
  }
}

function sizeAttrs(size) {
  return size ? ` width="${size.width}" height="${size.height}"` : ''
}
const outFile = join(siteRoot, 'src', 'generated', 'components.ts')
const repoBlob = 'https://github.com/mk7s/mkit/blob/main/book/src'

const catalog = JSON.parse(readFileSync(catalogPath, 'utf8'))
const registryEntries = JSON.parse(readFileSync(join(repoRoot, 'registry', 'registry.json'), 'utf8')).components
const registryNames = new Set(registryEntries.map((c) => c.name))
const registryByName = new Map(registryEntries.map((c) => [c.name, c]))

const keyLabel = (dispatch) => [...(dispatch.modifiers ?? []), dispatch.key].join('+')

// Summarize a registry entry's keyboard and accessibility contract from its
// generated conformance manifest (itself generated from the spec front matter).
function registryContract(name) {
  const entry = registryByName.get(name)
  const manifestPath = join(repoRoot, entry?.conformance_manifest ?? `registry/${name}/tests/conformance.json`)
  const contract = {
    name,
    status: entry?.status ?? '',
    spec: entry?.spec ?? '',
    states: [],
    keys: [],
    role: '',
    properties: [],
  }
  if (!existsSync(manifestPath)) return contract
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'))
  const seenKeys = new Set()
  for (const item of manifest.keyboard_cases ?? []) {
    const row = {
      keys: keyLabel(item.dispatch ?? {}),
      when: item.precondition ?? '',
      action: item.expected_behavior ?? '',
    }
    const id = `${row.keys}|${row.when}|${row.action}`
    if (seenKeys.has(id)) continue
    seenKeys.add(id)
    contract.keys.push(row)
  }
  const seenProps = new Set()
  for (const item of manifest.accessibility_cases ?? []) {
    if (item.state && !contract.states.includes(item.state)) contract.states.push(item.state)
    const semantics = item.expected_semantics ?? {}
    if (!contract.role && semantics.role) contract.role = semantics.role
    for (const property of semantics.properties ?? []) {
      const id = `${property.name}|${property.value}`
      if (seenProps.has(id)) continue
      seenProps.add(id)
      contract.properties.push({ name: String(property.name), value: String(property.value) })
    }
  }
  return contract
}

const slugify = (value) =>
  value
    .toLowerCase()
    .replace(/&/g, ' and ')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')

const escapeHtml = (value) =>
  String(value).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')

const escapeAttr = (value) => escapeHtml(value).replace(/"/g, '&quot;')

// Map every source chapter to the catalog card that owns it, so book links to a
// component chapter can become links to the card's page.
const cardBySource = new Map()
const componentSourceToCard = new Map()
for (const kind of Object.keys(catalog)) {
  for (const card of catalog[kind]) {
    for (const source of card.sources ?? []) {
      const key = `components/${source.replace(/^\.\//, '')}`
      cardBySource.set(key, card)
      componentSourceToCard.set(key, card)
    }
  }
}

const marked = new Marked({ gfm: true, breaks: false })
marked.use({
  renderer: {
    code(token) {
      const language = (token.lang || '').trim().split(/\s+/)[0]
      const source = token.text ?? ''
      let body
      if (language && hljs.getLanguage(language)) {
        try {
          body = hljs.highlight(source, { language, ignoreIllegals: true }).value
        } catch {
          body = escapeHtml(source)
        }
      } else {
        body = escapeHtml(source)
      }
      const label = language ? `<span class="code-lang">${escapeHtml(language)}</span>` : ''
      return `<div class="code-block">${label}<pre><code class="hljs">${body}</code></pre></div>`
    },
    link(token) {
      const text = this.parser.parseInline(token.tokens ?? [])
      const href = token.href || ''
      const external = /^https?:\/\//i.test(href)
      const attrs = external ? ' target="_blank" rel="noreferrer"' : ''
      return `<a href="${escapeAttr(href)}"${attrs}>${text}</a>`
    },
    image(token) {
      const alt = escapeAttr(token.text || '')
      return `<img src="${escapeAttr(token.href || '')}" alt="${alt}"${sizeAttrs(bookImageSize(token.href))} loading="lazy" />`
    },
  },
})

function shiftHeadings(html, offset) {
  return html.replace(/<(\/?)h([1-5])([ >])/g, (_, slash, level, tail) => {
    const next = Math.min(Number(level) + offset, 6)
    return `<${slash}h${next}${tail}`
  })
}

function anchorFromFile(file, anchor) {
  const lines = file.split(/\r?\n/)
  if (!anchor) return lines.join('\n')
  const start = lines.findIndex((line) => new RegExp(`^\\s*//\\s*ANCHOR:\\s*${anchor}\\s*$`).test(line))
  if (start === -1) return lines.join('\n')
  const end = lines.findIndex(
    (line, index) => index > start && new RegExp(`^\\s*//\\s*ANCHOR_END:\\s*${anchor}\\s*$`).test(line),
  )
  return lines.slice(start + 1, end === -1 ? undefined : end).join('\n')
}

// Replace mdBook {{#include path:anchor}} directives inside fenced code blocks
// with the referenced example source.
function resolveIncludes(markdown, sourceDir) {
  return markdown.replace(/```([^\n]*)\n([\s\S]*?)```/g, (whole, lang, body) => {
    if (!body.includes('{{#include')) return whole
    const resolved = body
      .split(/\r?\n/)
      .map((line) => {
        const match = line.match(/^\s*\{\{#include\s+([^}]+?)\}\}\s*$/)
        if (!match) return line
        const target = match[1].trim()
        const colon = target.lastIndexOf(':')
        const filePart = colon === -1 ? target : target.slice(0, colon)
        const anchor = colon === -1 ? '' : target.slice(colon + 1)
        const filePath = resolve(sourceDir, filePart)
        if (!existsSync(filePath)) return `// Missing include: ${target}`
        const contents = anchorFromFile(readFileSync(filePath, 'utf8'), anchor)
        return contents
      })
      .join('\n')
    return '```' + lang + '\n' + resolved + '\n```'
  })
}

// Rewrite relative image and link targets so the rendered HTML works in the
// site. Images are copied into public/book/ and referenced through a @book/
// placeholder that the client replaces with Vite's base URL.
function rewriteTargets(markdown, sourceDir, sourceKey, copiedImages) {
  const inFence = { value: false }
  return markdown
    .split(/\r?\n/)
    .map((line) => {
      if (/^\s*```/.test(line)) {
        inFence.value = !inFence.value
        return line
      }
      if (inFence.value) return line
      return line.replace(/(!?)\[([^\]]*)\]\(([^)\s]+)(\s+"[^"]*")?\)/g, (whole, bang, label, url, title) => {
        if (/^(https?:|mailto:|data:|#)/i.test(url)) return whole
        const clean = url.split('#')[0]
        const hash = url.includes('#') ? `#${url.split('#').slice(1).join('#')}` : ''
        if (!clean) return whole
        const abs = resolve(sourceDir, clean)
        if (bang === '!') {
          const relFromBook = relative(bookSrc, abs).split(sep).join('/')
          if (!relFromBook.startsWith('..') && existsSync(abs)) {
            const dest = join(publicBook, relFromBook)
            mkdirSync(dirname(dest), { recursive: true })
            if (!copiedImages.has(abs)) {
              copyFileSync(abs, dest)
              copiedImages.add(abs)
            }
            return `![${label}](@book/${relFromBook})`
          }
          return whole
        }
        // File links.
        if (/\.md$/i.test(clean)) {
          const relFromBook = relative(bookSrc, abs).split(sep).join('/')
          const card = componentSourceToCard.get(relFromBook)
          if (card) return `[${label}](/components/${card.slug}${hash})`
          if (!relFromBook.startsWith('..') && existsSync(abs)) {
            return `[${label}](${repoBlob}/${relFromBook}${hash})`
          }
          return whole
        }
        // Directory link such as `./` used from inside components/e8/.
        const relFromBook = relative(bookSrc, abs).split(sep).join('/')
        const owner = cardBySource.get(relative(bookSrc, abs).split(sep).join('/'))
        if (owner && !clean.includes('.')) return `[${label}](/components/${owner.slug}${hash})`
        if (!relFromBook.startsWith('..') && existsSync(abs) && clean.startsWith('.')) {
          return `[${label}](${repoBlob}/${relFromBook}${hash})`
        }
        return whole
      })
    })
    .join('\n')
}

function renderInline(text) {
  return marked.parseInline(text.trim()).trim()
}

function renderBlocks(markdown) {
  const html = marked.parse(markdown.trim())
  return shiftHeadings(html.trim(), 1)
}

function extractDoc(relSource, card) {
  const abs = join(bookComponents, relSource)
  const raw = readFileSync(abs, 'utf8').replace(/\r\n/g, '\n')
  const sourceKey = `components/${relSource}`
  const sourceDir = dirname(abs)
  const copiedImages = new Set()

  let withIncludes = resolveIncludes(raw, sourceDir)
  withIncludes = rewriteTargets(withIncludes, sourceDir, sourceKey, copiedImages)

  const lines = withIncludes.split('\n')
  let title = card.name
  let start = 0
  if (/^#\s+/.test(lines[0] ?? '')) {
    title = lines[0].replace(/^#\s+/, '').trim()
    start = 1
  }
  let body = lines.slice(start).join('\n').trim()

  // Collect a leading draft note or other blockquote.
  const notes = []
  body = body.replace(/^(?:>\s?.*(?:\n|$))+/, (block) => {
    const text = block
      .split('\n')
      .map((line) => line.replace(/^>\s?/, ''))
      .join('\n')
      .trim()
    if (text) notes.push(renderInline(text.replace(/\n+/g, ' ')))
    return ''
  }).trim()

  // Pull the first screenshot out as the component's hero image. A caption is
  // an italic-only line immediately after the image.
  let image
  const bodyLines = body.split('\n')
  const imageIndex = bodyLines.findIndex((line) => /^\s*!\[[^\]]*\]\([^)]+\)\s*$/.test(line))
  if (imageIndex !== -1) {
    const parsed = bodyLines[imageIndex].match(/^\s*!\[([^\]]*)\]\(([^)]+)\)\s*$/)
    const alt = parsed?.[1] ?? ''
    const src = parsed?.[2] ?? ''
    bodyLines.splice(imageIndex, 1)
    let caption
    let cursor = imageIndex
    while (cursor < bodyLines.length && bodyLines[cursor].trim() === '') cursor += 1
    if (cursor < bodyLines.length && /^\s*\*[^*].*\*\s*$/.test(bodyLines[cursor])) {
      caption = bodyLines[cursor].trim().replace(/^\*|\*$/g, '').trim()
      bodyLines.splice(cursor, 1)
    }
    body = bodyLines.join('\n').trim()
    image = { src, alt, caption, ...bookImageSize(src) }
  }

  // Split into preamble and `## ` sections.
  const chunks = body.split(/^(?=##\s+)/m)
  const preamble = chunks.shift() ?? ''
  const sections = []
  for (const chunk of chunks) {
    const heading = chunk.match(/^##\s+(.+)\n?/)
    if (!heading) continue
    const sectionTitle = heading[1].trim()
    const sectionBody = chunk.slice(heading[0].length).trim()
    if (!sectionBody) continue
    sections.push({
      id: `${slugify(title)}--${slugify(sectionTitle)}`,
      title: sectionTitle,
      html: renderBlocks(sectionBody),
    })
  }

  const ledeParagraphs = preamble
    .split(/\n\s*\n/)
    .map((block) => block.trim())
    .filter(Boolean)
  const firstParagraph = ledeParagraphs.shift() ?? ''
  const lede = firstParagraph ? renderInline(firstParagraph) : ''
  const intro = ledeParagraphs.length ? renderBlocks(ledeParagraphs.join('\n\n')) : ''

  // Demo key and primary registry name follow the chapter file name; paged
  // chapters (timeline-t1, timeline-t3-t4, node-editor-n2) share one registry
  // component. A chapter that documents several entries (disclosure and
  // accordion) names each entry's spec path, so those are picked up too.
  const demoKey = relSource.split('/').pop().replace(/\.md$/, '')
  const candidate =
    demoKey === 'number-field-pilot' ? 'scrubbable-number-field' : demoKey.replace(/(?:-[tn]\d+)+$/, '')
  const registryNamesForDoc = []
  if (registryNames.has(candidate)) registryNamesForDoc.push(candidate)
  for (const match of raw.matchAll(/registry\/([a-z0-9-]+)\/spec\.md/g)) {
    if (registryNames.has(match[1]) && !registryNamesForDoc.includes(match[1])) registryNamesForDoc.push(match[1])
  }
  const registryName = registryNamesForDoc[0] ?? ''

  return {
    slug: card.slug,
    name: title,
    demoKey,
    registryName,
    registryNames: registryNamesForDoc,
    contracts: registryNamesForDoc.map(registryContract),
    lede,
    intro,
    notes,
    image,
    sections,
    sourcePath: sourceKey,
  }
}

// Render a standalone book chapter for the site (images stay inline).
function extractChapter(relSource, fallbackTitle) {
  const abs = join(bookSrc, relSource)
  const raw = readFileSync(abs, 'utf8').replace(/\r\n/g, '\n')
  const sourceDir = dirname(abs)
  const copiedImages = new Set()
  let text = resolveIncludes(raw, sourceDir)
  text = rewriteTargets(text, sourceDir, relSource, copiedImages)
  const lines = text.split('\n')
  let title = fallbackTitle
  let start = 0
  if (/^#\s+/.test(lines[0] ?? '')) {
    title = lines[0].replace(/^#\s+/, '').trim()
    start = 1
  }
  let body = lines.slice(start).join('\n').trim()
  const notes = []
  body = body
    .replace(/^(?:>\s?.*(?:\n|$))+/, (block) => {
      const value = block
        .split('\n')
        .map((line) => line.replace(/^>\s?/, ''))
        .join('\n')
        .trim()
      if (value) notes.push(renderInline(value.replace(/\n+/g, ' ')))
      return ''
    })
    .trim()
  const chunks = body.split(/^(?=##\s+)/m)
  const preamble = (chunks.shift() ?? '').trim()
  const sections = []
  for (const chunk of chunks) {
    const heading = chunk.match(/^##\s+(.+)\n?/)
    if (!heading) continue
    const sectionTitle = heading[1].trim()
    const sectionBody = chunk.slice(heading[0].length).trim()
    if (!sectionBody) continue
    sections.push({
      id: `theming--${slugify(sectionTitle)}`,
      title: sectionTitle,
      html: renderBlocks(sectionBody),
    })
  }
  const paragraphs = preamble
    .split(/\n\s*\n/)
    .map((block) => block.trim())
    .filter(Boolean)
  const first = paragraphs.shift() ?? ''
  return {
    slug: 'theming',
    title,
    lede: first ? renderInline(first) : '',
    intro: paragraphs.length ? renderBlocks(paragraphs.join('\n\n')) : '',
    notes,
    sections,
    sourcePath: relSource,
  }
}

const generatedCards = []
for (const kind of Object.keys(catalog)) {
  for (const card of catalog[kind]) {
    const components = (card.sources ?? []).map((source) => extractDoc(source, card))
    if (!components.length) {
      components.push({
        slug: card.slug,
        name: card.name,
        lede: renderInline(card.note ?? card.description),
        intro: '',
        notes: [],
        image: undefined,
        sections: [],
        sourcePath: '',
        demoKey: '',
        registryName: '',
        registryNames: [],
        contracts: [],
      })
    }
    generatedCards.push({
      slug: card.slug,
      name: card.name,
      kind: kind === 'pro' ? 'pro' : 'everyday',
      group: card.group,
      description: card.description,
      status: card.status,
      tags: card.tags ?? [],
      preview: card.preview,
      accent: card.accent,
      components,
    })
  }
}

const theming = extractChapter('theming/design-language.md', 'The mkit design language')

// Clean out stale screenshots and copy every image the generated HTML needs.
rmSync(publicBook, { recursive: true, force: true })
const htmlParts = []
for (const card of generatedCards) {
  for (const component of card.components) {
    if (component.image?.src) htmlParts.push(component.image.src)
    htmlParts.push(component.lede, component.intro, ...component.notes)
    for (const section of component.sections) htmlParts.push(section.html)
  }
}
htmlParts.push(theming.lede, theming.intro, ...theming.notes)
for (const section of theming.sections) htmlParts.push(section.html)

const imageRefs = new Set()
for (const html of htmlParts) {
  for (const match of html.matchAll(/@book\/([^"'\s)]+)/g)) imageRefs.add(match[1])
}
for (const rel of imageRefs) {
  const from = join(bookSrc, rel)
  const to = join(publicBook, rel)
  if (existsSync(from)) {
    mkdirSync(dirname(to), { recursive: true })
    copyFileSync(from, to)
  }
}

mkdirSync(dirname(outFile), { recursive: true })
const banner = `// AUTO-GENERATED — do not edit.\n// Source: book/src/components/**/*.md and book/src/theming/design-language.md\n// via scripts/generate-content.mjs. Run \`npm run content\` after changing the book.\n\nimport type { CatalogCard, ChapterDoc } from '../types'\n\n`
const data = JSON.stringify(generatedCards, null, 2)
const chapter = JSON.stringify(theming, null, 2)
writeFileSync(
  outFile,
  `${banner}export const catalog: CatalogCard[] = ${data}\n\nexport const theming: ChapterDoc = ${chapter}\n`,
)

const chapterCount = generatedCards.reduce((total, card) => total + card.components.length, 0)
const documented = new Set(
  generatedCards.flatMap((card) => card.components.flatMap((component) => component.registryNames)),
)
const undocumented = [...registryNames].filter((name) => !documented.has(name))
if (undocumented.length) {
  console.warn(`content: registry components without a site page: ${undocumented.join(', ')}`)
}
console.log(
  `content: wrote ${generatedCards.length} cards from ${chapterCount} book chapters (${documented.size}/${registryNames.size} registry components), plus the theming chapter`,
)
