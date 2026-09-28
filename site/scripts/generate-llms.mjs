// Build-time generator for the agent-facing text exports of the mkit site.
//
// Writes two files into public/ so Vite copies them to the site root:
//
// - llms.txt       An index that follows the llms.txt convention (https://llmstxt.org):
//                  book chapters grouped by SUMMARY.md part, the site component
//                  pages from content/catalog.json, and optional material.
// - llms-full.txt  The full text of The GPUI Book in SUMMARY.md order, with mdBook
//                  {{#include}} directives resolved to the example sources, images
//                  replaced by their alt text, and the registry status appended.
//
// Output is deterministic (no timestamps), so repeated builds do not churn.
// Uses Node built-ins only.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, join, relative, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const siteRoot = resolve(here, '..')
const repoRoot = resolve(siteRoot, '..')
const bookSrc = join(repoRoot, 'book', 'src')
const summaryPath = join(bookSrc, 'SUMMARY.md')
const catalogPath = join(siteRoot, 'content', 'catalog.json')
const registryPath = join(repoRoot, 'registry', 'registry.json')
const publicDir = join(siteRoot, 'public')
const indexOut = join(publicDir, 'llms.txt')
const fullOut = join(publicDir, 'llms-full.txt')
const FULL_URL = 'llms-full.txt'

const toPosix = (value) => value.split(sep).join('/')
const repoRel = (abs) => toPosix(relative(repoRoot, abs))

const catalog = JSON.parse(readFileSync(catalogPath, 'utf8'))
const registry = JSON.parse(readFileSync(registryPath, 'utf8'))

// ---------------------------------------------------------------------------
// SUMMARY.md parsing

function parseSummary(markdown) {
  const chapters = []
  let part = ''
  for (const line of markdown.replace(/\r\n/g, '\n').split('\n')) {
    const partMatch = line.match(/^#\s+(Part\s+.+?)\s*$/)
    if (partMatch) {
      part = partMatch[1]
      continue
    }
    const item = line.match(/^(\s*)[-*]\s+\[([^\]]+)\]\(([^)]*)\)/)
    if (!item) continue
    const path = item[3].trim()
    if (!path) continue // draft chapter without a file
    chapters.push({
      title: item[2].trim(),
      path,
      depth: Math.floor(item[1].replace(/\t/g, '  ').length / 2),
      part,
    })
  }
  return chapters
}

// ---------------------------------------------------------------------------
// mdBook include resolution (mirrors resolveIncludes/anchorFromFile in
// generate-content.mjs, extended to line ranges and includes outside fences)

const anchorMarker = /^\s*(?:\/\/|#|<!--)\s*ANCHOR(?:_END)?:\s*[\w-]+/
const escapeRegExp = (value) => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

function anchorFromFile(file, anchor) {
  const lines = file.replace(/\r\n/g, '\n').split('\n')
  if (!anchor) return lines.join('\n')
  const range = anchor.match(/^(\d*)(?::(\d*))?$/)
  if (range && (range[1] || range[2] !== undefined)) {
    // mdBook line ranges: `:N` (one line), `N:` (N to end), `:M` via `::M`, `N:M`.
    const from = range[1] ? Number(range[1]) : 1
    const to = range[2] === undefined ? from : range[2] === '' ? lines.length : Number(range[2])
    return lines.slice(from - 1, to).join('\n')
  }
  const name = escapeRegExp(anchor)
  const start = lines.findIndex((line) => new RegExp(`^\\s*//\\s*ANCHOR:\\s*${name}\\s*$`).test(line))
  if (start === -1) return null
  const end = lines.findIndex(
    (line, index) => index > start && new RegExp(`^\\s*//\\s*ANCHOR_END:\\s*${name}\\s*$`).test(line),
  )
  return lines.slice(start + 1, end === -1 ? undefined : end).join('\n')
}

const missingIncludes = []

function resolveIncludes(markdown, sourceDir, sourceRel) {
  const out = []
  let fence = null
  for (const line of markdown.split('\n')) {
    const fenceMatch = line.match(/^\s*(`{3,}|~{3,})/)
    if (fenceMatch) {
      const marker = fenceMatch[1]
      if (!fence) fence = marker
      else if (marker[0] === fence[0] && marker.length >= fence.length) fence = null
      out.push(line)
      continue
    }
    const match = line.match(/^\s*\{\{#include\s+([^}]+?)\}\}\s*$/)
    if (!match) {
      out.push(line)
      continue
    }
    const target = match[1].trim()
    const colon = target.indexOf(':')
    const filePart = colon === -1 ? target : target.slice(0, colon)
    const anchor = colon === -1 ? '' : target.slice(colon + 1)
    const filePath = resolve(sourceDir, filePart)
    const contents = existsSync(filePath) ? anchorFromFile(readFileSync(filePath, 'utf8'), anchor) : null
    if (contents === null) {
      missingIncludes.push(`${sourceRel}: ${target}`)
      out.push(fence ? `// Missing include: ${target}` : `<!-- Missing include: ${target} -->`)
      continue
    }
    const kept = contents.split('\n').filter((l) => !anchorMarker.test(l))
    while (kept.length && kept[kept.length - 1].trim() === '') kept.pop()
    out.push(...kept)
  }
  return out.join('\n')
}

// Replace image embeds with their alt text; leave code fences untouched.
function dropImages(markdown) {
  let fence = null
  return markdown
    .split('\n')
    .map((line) => {
      const fenceMatch = line.match(/^\s*(`{3,}|~{3,})/)
      if (fenceMatch) {
        const marker = fenceMatch[1]
        if (!fence) fence = marker
        else if (marker[0] === fence[0] && marker.length >= fence.length) fence = null
        return line
      }
      if (fence) return line
      return line
        .replace(/!\[([^\]]*)\]\([^)]*\)/g, (_, alt) => (alt.trim() ? `[Image: ${alt.trim()}]` : ''))
        .replace(/<img\b[^>]*?\balt="([^"]*)"[^>]*>/gi, (_, alt) => (alt.trim() ? `[Image: ${alt.trim()}]` : ''))
        .replace(/<img\b[^>]*>/gi, '')
    })
    .join('\n')
}

// First sentence of the first prose paragraph, for index descriptions.
function describe(markdown) {
  let fence = false
  const paragraphs = []
  let current = []
  for (const line of markdown.split('\n')) {
    if (/^\s*(```|~~~)/.test(line)) {
      fence = !fence
      if (current.length) paragraphs.push(current.join(' '))
      current = []
      continue
    }
    if (fence) continue
    if (!line.trim() || /^\s*(#|>|\||[-*+]\s|\d+\.\s|<|\[Image:|\{\{)/.test(line)) {
      if (current.length) paragraphs.push(current.join(' '))
      current = []
      continue
    }
    current.push(line.trim())
  }
  if (current.length) paragraphs.push(current.join(' '))
  const first = paragraphs.find((p) => /[a-z]/i.test(p))
  if (!first) return ''
  const plain = first
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/[*_]{1,2}([^*_]+)[*_]{1,2}/g, '$1')
    .replace(/\s+/g, ' ')
    .trim()
  let sentence = plain.match(/^.+?[.!?](?=\s|$)/)?.[0] ?? plain
  if (sentence.length < 25) sentence = plain
  return sentence.length > 200 ? `${sentence.slice(0, 197).trimEnd()}...` : sentence
}

// ---------------------------------------------------------------------------
// Load chapters

const chapters = parseSummary(readFileSync(summaryPath, 'utf8'))
const missingChapters = []
for (const chapter of chapters) {
  const abs = join(bookSrc, chapter.path)
  chapter.source = repoRel(abs)
  if (!existsSync(abs)) {
    missingChapters.push(chapter.source)
    console.warn(`llms: warning: chapter listed in SUMMARY.md is missing: ${chapter.source}`)
    chapter.body = null
    continue
  }
  const raw = readFileSync(abs, 'utf8').replace(/\r\n/g, '\n')
  chapter.body = dropImages(resolveIncludes(raw, dirname(abs), chapter.source)).trim()
  const withoutTitle = chapter.body.replace(/^#\s+.*\n?/, '')
  chapter.description = describe(withoutTitle)
}

const isOptional = (chapter) => chapter.path.startsWith('appendices/')

// ---------------------------------------------------------------------------
// llms-full.txt

const full = []
full.push('# mkit and The GPUI Book: full text')
full.push('')
full.push(
  '> The complete text of The GPUI Book in SUMMARY.md order, followed by the mkit component registry. ' +
    'Code samples are resolved from the compiling example crates through the book\'s mdBook `{{#include}}` anchors. ' +
    'Images are replaced by their alt text.',
)
full.push('')
const statuses = [...new Set(registry.components.map((c) => c.status))].sort()
const statusList = statuses.map((s) => `\`${s}\``).join(', ')
full.push(
  `Status: pre-release. All ${registry.components.length} registry components have status ${statusList} in \`registry/registry.json\` (see the table at the end); none is released or stable. ` +
    'Relative links inside chapters are relative to the chapter\'s source path in the mkit repository.',
)
chapters.forEach((chapter, index) => {
  full.push('')
  full.push('---')
  full.push('')
  full.push(`<!-- source: ${chapter.source} -->`)
  const where = chapter.part ? ` · ${chapter.part}` : ''
  full.push(`**Chapter ${index + 1}: ${chapter.title}${where} · Source: \`${chapter.source}\`**`)
  full.push('')
  full.push(chapter.body ?? `_Missing chapter file: \`${chapter.source}\`._`)
})

full.push('')
full.push('---')
full.push('')
full.push('<!-- source: registry/registry.json -->')
full.push('# Component registry')
full.push('')
full.push(
  `Registry format version ${registry.registry_format_version}, release version ${registry.release_version}. ` +
    `Distribution: ${registry.distribution?.kind ?? 'unknown'} at \`${registry.distribution?.path ?? ''}\`, ` +
    `published: ${registry.distribution?.published === true ? 'true' : 'false'}. ` +
    'Statuses below are copied verbatim from `registry/registry.json`.',
)
full.push('')
full.push('| Component | Version | Status | Spec |')
full.push('| --- | --- | --- | --- |')
for (const component of registry.components) {
  full.push(`| ${component.name} | ${component.version} | ${component.status} | \`${component.spec}\` |`)
}
full.push('')
const fullText = full.join('\n')

// ---------------------------------------------------------------------------
// llms.txt

let linkCount = 0
const link = (title, url, description) => {
  linkCount += 1
  return `- [${title}](${url})${description ? `: ${description}` : ''}`
}
const chapterLink = (chapter) => {
  const indent = '  '.repeat(chapter.depth)
  const note = chapter.description ? `${chapter.description} ` : ''
  return `${indent}${link(chapter.title, FULL_URL, `${note}(\`${chapter.source}\`)`)}`
}

const index = []
index.push('# mkit')
index.push('')
index.push(
  '> mkit is a component kit for GPUI, the Rust UI framework from Zed, together with The GPUI Book: ' +
    'a guide to building desktop apps with GPUI and a registry of copy-in component sources that depend only on `mkit-core` and GPUI.',
)
index.push('')
index.push(
  `Status: pre-release. All ${registry.components.length} registry components have status ` +
    `${statusList} in \`registry/registry.json\`; none is released or stable${registry.distribution?.published === true ? '' : ', and the registry is not published'}. ` +
    'Book code samples compile from the `examples/` crates and are pulled into chapters with mdBook `{{#include}}` anchors. ' +
    `Book chapters are not hosted as individual pages yet, so chapter links below point to [${FULL_URL}](${FULL_URL}), ` +
    'which holds the full book text in order with a `<!-- source: book/src/... -->` marker before each chapter.',
)

index.push('')
index.push('## The GPUI Book')
index.push('')
index.push(link('Full text of The GPUI Book', FULL_URL, 'every chapter in SUMMARY.md order, with example code resolved, plus the component registry status'))
let currentPart = null
for (const chapter of chapters) {
  if (isOptional(chapter) || chapter.body === null) continue
  if (chapter.part !== currentPart) {
    currentPart = chapter.part
    if (currentPart) {
      index.push('')
      index.push(`## The GPUI Book, ${currentPart}`)
      index.push('')
    }
  }
  index.push(chapterLink(chapter))
}

index.push('')
index.push('## Components')
index.push('')
for (const kind of Object.keys(catalog)) {
  for (const card of catalog[kind]) {
    index.push(link(card.name, `components/${card.slug}`, card.description))
  }
}
const theming = chapters.find((c) => c.path === 'theming/design-language.md')
index.push(link('Theming and design language', 'theming', theming?.description))

index.push('')
index.push('## Optional')
index.push('')
index.push(link('Full text export', FULL_URL, 'the whole book as one Markdown file, ending with the component registry table'))
for (const chapter of chapters) {
  if (!isOptional(chapter) || chapter.body === null) continue
  index.push(chapterLink(chapter))
}
index.push('')
const indexText = index.join('\n')

// ---------------------------------------------------------------------------
// Write

mkdirSync(publicDir, { recursive: true })
writeFileSync(indexOut, indexText)
writeFileSync(fullOut, fullText)

if (missingIncludes.length) {
  console.warn(`llms: warning: ${missingIncludes.length} missing include(s):`)
  for (const entry of missingIncludes) console.warn(`  ${entry}`)
}
const writtenChapters = chapters.filter((c) => c.body !== null).length
const kb = Math.round(Buffer.byteLength(fullText, 'utf8') / 1024)
console.log(
  `llms: wrote llms.txt (${linkCount} links) and llms-full.txt (${writtenChapters} chapters, ${kb} KB)` +
    (missingIncludes.length ? `, ${missingIncludes.length} missing includes` : '') +
    (missingChapters.length ? `, ${missingChapters.length} missing chapters` : ''),
)
