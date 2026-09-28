// Build-time book-health data for the mkit component site (plan item E12.1).
//
// Reads repository files (book/src, examples/, registry/, evals/) and writes
// src/generated/health.ts. Every number is computed from the repo; nothing is
// hard-coded. Output is deterministic apart from the commit and check result.
//
// Usage:
//   node scripts/generate-health.mjs              # check.status = 'not-run'
//   node scripts/generate-health.mjs --run-check  # also runs the book checker (~20 s)
//
// A failing checker is recorded as status 'fail' with a warning; it never fails
// the site build.

import { execFileSync, spawnSync } from 'node:child_process'
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs'
import { dirname, join, relative, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const siteRoot = resolve(here, '..')
const repoRoot = resolve(siteRoot, '..')
const bookSrc = join(repoRoot, 'book', 'src')
const outFile = join(siteRoot, 'src', 'generated', 'health.ts')

const CHECK_COMMAND = 'python3 scripts/check_book.py --skip-cargo'
const runCheck = process.argv.includes('--run-check')

const toPosix = (path) => path.split(sep).join('/')

// Recursively list files under `dir`, sorted, as absolute paths.
function walk(dir) {
  if (!existsSync(dir)) return []
  const out = []
  for (const entry of readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const full = join(dir, entry.name)
    if (entry.isDirectory()) out.push(...walk(full))
    else if (entry.isFile()) out.push(full)
  }
  return out
}

// --- commit -----------------------------------------------------------------

function gitCommit() {
  try {
    return execFileSync('git', ['rev-parse', '--short', 'HEAD'], {
      cwd: repoRoot,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim()
  } catch {
    return ''
  }
}

// --- book checker -------------------------------------------------------------

function bookCheck() {
  if (!runCheck) {
    return {
      command: CHECK_COMMAND,
      status: 'not-run',
      summary: 'Checker not run for this build. Run `npm run build` to record a checker result.',
    }
  }
  const result = spawnSync('python3', ['scripts/check_book.py', '--skip-cargo'], {
    cwd: repoRoot,
    encoding: 'utf8',
    timeout: 120_000,
  })
  const output = `${result.stdout ?? ''}\n${result.stderr ?? ''}`
  const lastLine =
    output
      .split('\n')
      .map((line) => line.trim())
      .filter(Boolean)
      .pop() ?? ''
  if (result.error) {
    const summary = `Checker could not run: ${result.error.code ?? result.error.message}`
    console.warn(`generate-health: warning: ${summary}`)
    return { command: CHECK_COMMAND, status: 'fail', summary }
  }
  if (result.status === 0) {
    return { command: CHECK_COMMAND, status: 'pass', summary: lastLine || 'Checker exited 0 with no output.' }
  }
  const summary = lastLine || `Checker exited with ${result.status ?? result.signal}.`
  console.warn(`generate-health: warning: book checker failed: ${summary}`)
  return { command: CHECK_COMMAND, status: 'fail', summary }
}

// --- SUMMARY.md -----------------------------------------------------------------

function summary() {
  const text = readFileSync(join(bookSrc, 'SUMMARY.md'), 'utf8')
  const parts = []
  const links = new Set()
  let current = null
  for (const line of text.split('\n')) {
    const heading = line.match(/^#\s+(Part\b.*?)\s*$/)
    if (heading) {
      current = { title: heading[1], chapters: 0 }
      parts.push(current)
      continue
    }
    const item = line.match(/^\s*[-*]\s+\[[^\]]*\]\(([^)\s]+)\)/)
    if (!item) continue
    if (!current) {
      current = { title: 'Introduction', chapters: 0 }
      parts.push(current)
    }
    current.chapters += 1
    links.add(item[1].replace(/^\.\//, ''))
  }
  return {
    parts,
    inSummary: links.size,
    componentChapters: [...links].filter((link) => link.startsWith('components/')).length,
  }
}

// --- includes ---------------------------------------------------------------------

// Count mdBook include directives that stand on their own line (the form the
// checker accepts), so prose that mentions `{{#include}}` is not counted.
function includes(markdownFiles) {
  let total = 0
  let pages = 0
  for (const file of markdownFiles) {
    const count = (readFileSync(file, 'utf8').match(/^[ \t]*\{\{#include\s+[^}]+\}\}[ \t]*$/gm) ?? []).length
    total += count
    if (count > 0) pages += 1
  }
  return { includes: total, pagesWithIncludes: pages }
}

function exampleCrates() {
  const examples = join(repoRoot, 'examples')
  if (!existsSync(examples)) return 0
  return readdirSync(examples, { withFileTypes: true }).filter(
    (entry) => entry.isDirectory() && existsSync(join(examples, entry.name, 'Cargo.toml')),
  ).length
}

// --- concept index ------------------------------------------------------------------

function conceptIndex() {
  const text = readFileSync(join(bookSrc, 'appendices', 'concept-index.md'), 'utf8')
  const groups = []
  let group = null
  let total = 0
  let covered = 0
  let future = 0
  for (const line of text.split('\n')) {
    const heading = line.match(/^##\s+(.*?)\s*$/)
    if (heading) {
      const title = heading[1]
        .replace(/^E\d+(?:\.\d+)*:\s*/, '') // "E2.4: Getting started (5)" -> "Getting started (5)"
        .replace(/\s*\(\d+\)\s*$/, '') // drop the hand-written count; we compute it
      group = { title, total: 0, covered: 0 }
      groups.push(group)
      continue
    }
    if (!group || !line.trim().startsWith('|')) continue
    const cells = line
      .trim()
      .replace(/^\|/, '')
      .replace(/\|$/, '')
      .split('|')
      .map((cell) => cell.trim())
    // Skip the header row and the separator row.
    if (cells.every((cell) => /^:?-{3,}:?$/.test(cell))) continue
    if (cells[0] === 'Public item') continue
    const route = cells[cells.length - 1]
    group.total += 1
    total += 1
    if (route === 'Future coverage') future += 1
    else if (/\[[^\]]*\]\([^)]+\)/.test(route)) {
      group.covered += 1
      covered += 1
    }
  }
  return { total, covered, future, groups }
}

// --- registry -------------------------------------------------------------------------

function registry() {
  const data = JSON.parse(readFileSync(join(repoRoot, 'registry', 'registry.json'), 'utf8'))
  const components = data.components ?? []
  const counts = new Map()
  for (const component of components) {
    const status = component.status ?? 'unknown'
    counts.set(status, (counts.get(status) ?? 0) + 1)
  }
  const statuses = [...counts]
    .map(([status, count]) => ({ status, count }))
    .sort((a, b) => b.count - a.count || a.status.localeCompare(b.status))
  const withSpec = components.filter(
    (component) => component.spec && existsSync(join(repoRoot, component.spec)) && statSync(join(repoRoot, component.spec)).isFile(),
  ).length
  return { components: components.length, statuses, withSpec }
}

// --- evaluations ----------------------------------------------------------------------

function evaluations() {
  const evals = join(repoRoot, 'evals')
  const readme = join(evals, 'README.md')
  // A benchmark task is a directory under evals/benchmarks with a spec.md.
  const benchmarks = join(evals, 'benchmarks')
  const tasks = existsSync(benchmarks)
    ? walk(benchmarks).filter((file) => /^[^/]+\/spec\.md$/.test(toPosix(relative(benchmarks, file)))).length
    : 0
  let note = ''
  if (existsSync(readme)) {
    const paragraph = []
    for (const line of readFileSync(readme, 'utf8').split('\n')) {
      if (!paragraph.length && (line.trim() === '' || /^#/.test(line))) continue
      if (line.trim() === '') break
      paragraph.push(line)
    }
    note = paragraph.join('\n')
  }
  return { status: 'pending', tasks, note }
}

// --- assemble -------------------------------------------------------------------------

const markdownFiles = walk(bookSrc).filter((file) => file.endsWith('.md'))
const summaryData = summary()
const includeData = includes(markdownFiles)
const images = walk(join(bookSrc, 'images')).filter((file) => /\.(png|jpe?g|svg)$/i.test(file)).length

const bookHealth = {
  commit: gitCommit(),
  check: bookCheck(),
  parts: summaryData.parts,
  pages: {
    markdown: markdownFiles.length,
    inSummary: summaryData.inSummary,
    componentChapters: summaryData.componentChapters,
  },
  examples: {
    crates: exampleCrates(),
    includes: includeData.includes,
    pagesWithIncludes: includeData.pagesWithIncludes,
  },
  screenshots: { images },
  conceptIndex: conceptIndex(),
  registry: registry(),
  evaluations: evaluations(),
}

const source = `// AUTO-GENERATED — do not edit.
// Source: book/src, examples/, registry/registry.json and evals/
// via scripts/generate-health.mjs. Pass --run-check to record a book checker result.

export type BookHealth = {
  commit: string
  check: {
    command: string
    status: 'pass' | 'fail' | 'not-run'
    summary: string
  }
  parts: { title: string; chapters: number }[]
  pages: { markdown: number; inSummary: number; componentChapters: number }
  examples: { crates: number; includes: number; pagesWithIncludes: number }
  screenshots: { images: number }
  conceptIndex: {
    total: number
    covered: number
    future: number
    groups: { title: string; total: number; covered: number }[]
  }
  registry: { components: number; statuses: { status: string; count: number }[]; withSpec: number }
  evaluations: { status: 'pending'; tasks: number; note: string }
}

export const bookHealth: BookHealth = ${JSON.stringify(bookHealth, null, 2)}
`

mkdirSync(dirname(outFile), { recursive: true })
writeFileSync(outFile, source)

const ci = bookHealth.conceptIndex
console.log(
  `generate-health: ${bookHealth.pages.markdown} pages, ${bookHealth.examples.crates} example crates, ` +
    `${ci.covered}/${ci.total} concepts covered, check ${bookHealth.check.status} -> ${toPosix(relative(siteRoot, outFile))}`,
)
