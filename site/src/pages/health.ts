import { bookHealth } from '../generated/health'
import { icon } from '../icons'
import { bookBlobUrl, siteHref } from '../links'
import { docsSidebar, docsTopbar, mountDocsNav } from './docs'

// Book health (plan item E12.1): what the build can verify about the book
// today. Every number comes from src/generated/health.ts, which
// scripts/generate-health.mjs computes from repository files.

const percent = (part: number, whole: number) => (whole ? Math.round((part / whole) * 100) : 0)

const escapeHtml = (value: string) =>
  value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')

const inlineCode = (value: string) => escapeHtml(value).replace(/`([^`]+)`/g, '<code>$1</code>')

const checkLabel = { pass: 'Passing', fail: 'Failing', 'not-run': 'Not run' } as const

function meter(value: number, max: number, label: string) {
  const width = percent(value, max)
  return `<span class="health-meter" role="img" aria-label="${label}"><i style="width:${width}%"></i></span>`
}

function tile(label: string, value: string, note: string, tone = '') {
  return `<div class="health-tile${tone ? ` ${tone}` : ''}">
    <span class="health-tile-label">${label}</span>
    <b>${value}</b>
    <span class="health-tile-note">${note}</span>
  </div>`
}

export function renderHealth() {
  const h = bookHealth
  const inScope = h.conceptIndex.groups.filter((group) => !/^excluded/i.test(group.title))
  const excluded = h.conceptIndex.groups.filter((group) => /^excluded/i.test(group.title))
  const scopeTotal = inScope.reduce((sum, group) => sum + group.total, 0)
  const scopeCovered = inScope.reduce((sum, group) => sum + group.covered, 0)
  const excludedTotal = excluded.reduce((sum, group) => sum + group.total, 0)
  const maxChapters = Math.max(...h.parts.map((part) => part.chapters), 1)
  const checkTone = h.check.status === 'pass' ? 'good' : h.check.status === 'fail' ? 'bad' : 'muted'

  const parts = h.parts
    .map(
      (part) => `<tr>
        <th scope="row">${escapeHtml(part.title)}</th>
        <td class="num">${part.chapters}</td>
        <td>${meter(part.chapters, maxChapters, `${part.chapters} chapters`)}</td>
      </tr>`,
    )
    .join('')

  const concepts = inScope
    .map(
      (group) => `<tr>
        <th scope="row">${escapeHtml(group.title)}</th>
        <td class="num">${group.covered} / ${group.total}</td>
        <td>${meter(group.covered, group.total, `${percent(group.covered, group.total)}% covered`)}</td>
      </tr>`,
    )
    .join('')

  const statuses = h.registry.statuses
    .map((row) => `<li><code>${escapeHtml(row.status)}</code>: ${row.count}</li>`)
    .join('')

  return `<div class="docs-shell">
    ${docsTopbar('Book health')}
    <div class="docs-layout">
      ${docsSidebar('health')}
      <main class="docs-main" id="main" tabindex="-1">
        <nav class="docs-breadcrumb" aria-label="Breadcrumb">
          <a href="${siteHref()}">Home</a><span aria-hidden="true">/</span><span>Guides</span><span aria-hidden="true">/</span><b aria-current="page">Book health</b>
        </nav>
        <header class="docs-header">
          <span class="docs-kicker">The GPUI Book</span>
          <h1>Book health</h1>
          <p class="docs-lede">What the build can check about the book today: the checker result, what the book contains, and how much of GPUI's public API it teaches. The numbers are generated from the repository when the site is built${h.commit ? ` at commit <code>${escapeHtml(h.commit)}</code>` : ''}.</p>
          <div class="docs-callout">${icon('book', 15)}<span><b>Agent evaluations are pending.</b> ${inlineCode(h.evaluations.note)} Success rates will appear here once evaluations run.</span></div>
        </header>
        <article class="docs-article health">
          <section class="health-tiles" aria-label="Summary">
            ${tile('Book check', checkLabel[h.check.status], h.check.status === 'not-run' ? 'Recorded by production builds' : 'Structure, includes, links, screenshots', checkTone)}
            ${tile('Chapters', String(h.pages.inSummary), `${h.pages.componentChapters} component chapters`)}
            ${tile('Example crates', String(h.examples.crates), `${h.examples.includes} code includes on ${h.examples.pagesWithIncludes} pages`)}
            ${tile('Screenshots', String(h.screenshots.images), 'Images under book/src/images')}
            ${tile('API coverage', `${percent(scopeCovered, scopeTotal)}%`, `${scopeCovered} of ${scopeTotal} public items routed`)}
            ${tile('Evaluations', 'Pending', `${h.evaluations.tasks} benchmark ${h.evaluations.tasks === 1 ? 'task' : 'tasks'} defined, no results yet`, 'muted')}
          </section>

          <section class="doc-section" id="health-check">
            <h3>Book checker</h3>
            <p>The checker verifies mdBook includes and anchors, local links, screenshot provenance, and that the concept index is current. Rust examples are compiled separately by the workspace build, so this run skips Cargo.</p>
            <div class="health-check ${checkTone}">
              <span class="status-pill">${checkLabel[h.check.status]}</span>
              <code>${escapeHtml(h.check.command)}</code>
              <p>${inlineCode(h.check.summary)}</p>
            </div>
          </section>

          <section class="doc-section" id="health-contents">
            <h3>What the book contains</h3>
            <p>${h.pages.inSummary} chapters are listed in the table of contents, out of ${h.pages.markdown} Markdown files in <code>book/src</code>. Code samples come from ${h.examples.crates} example crates through ${h.examples.includes} <code>{{#include}}</code> anchors, so every Rust block on a page is compiled.</p>
            <table class="health-table">
              <caption class="sr-only">Chapters per part</caption>
              <thead><tr><th scope="col">Part</th><th scope="col" class="num">Chapters</th><th scope="col"><span class="sr-only">Relative size</span></th></tr></thead>
              <tbody>${parts}</tbody>
            </table>
          </section>

          <section class="doc-section" id="health-concepts">
            <h3>Concept index coverage</h3>
            <p>The <a href="${bookBlobUrl}/appendices/concept-index.md" target="_blank" rel="noreferrer">concept index<span class="sr-only"> (opens in a new tab)</span></a> lists all ${h.conceptIndex.total} public items in the pinned GPUI inventory. ${h.conceptIndex.covered} have a published chapter that introduces them; ${h.conceptIndex.future} are marked future coverage. ${excludedTotal} items in ${excluded.length} excluded groups (hidden implementation support and re-exports) are left out of the percentage below.</p>
            <table class="health-table">
              <caption class="sr-only">Public API items with a teaching chapter, by group</caption>
              <thead><tr><th scope="col">Group</th><th scope="col" class="num">Routed</th><th scope="col"><span class="sr-only">Coverage</span></th></tr></thead>
              <tbody>${concepts}</tbody>
            </table>
          </section>

          <section class="doc-section" id="health-registry">
            <h3>Component registry</h3>
            <p>${h.registry.components} registry entries, ${h.registry.withSpec} with a spec. No entry is <code>source_ready</code> yet, so none can be installed with <code>cargo mkit add</code>.</p>
            <ul>${statuses}</ul>
          </section>

          <section class="doc-section" id="health-agents">
            <h3>Docs for agents</h3>
            <p>The build also writes plain-text exports for language models: <a href="${siteHref('llms.txt')}" data-native>llms.txt</a>, an index of the book and component pages, and <a href="${siteHref('llms-full.txt')}" data-native>llms-full.txt</a>, the full text of every chapter with example code resolved.</p>
          </section>
        </article>
      </main>
      <aside class="docs-toc" aria-label="On this page">
        <span class="docs-toc-heading">On this page</span>
        <a class="docs-toc-link" href="#health-check">Book checker</a>
        <a class="docs-toc-link" href="#health-contents">What the book contains</a>
        <a class="docs-toc-link" href="#health-concepts">Concept index coverage</a>
        <a class="docs-toc-link" href="#health-registry">Component registry</a>
        <a class="docs-toc-link" href="#health-agents">Docs for agents</a>
      </aside>
    </div>
  </div>`
}

export function mountHealth() {
  return mountDocsNav()
}
