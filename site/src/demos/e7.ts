import { ui } from '../ui/icons'
import type { DemoMap } from '../ui/types'

const box = (label: string, checked: boolean | 'mixed' = false) =>
  `<button class="ui-check" role="checkbox" aria-checked="${checked}" aria-label="${label}">${ui('check')}${ui('minus')}</button>`

const check = (label: string, checked: boolean | 'mixed' = false) => `<label class="ui-choice">${box(label, checked)}${label}</label>`

const radio = (id: string, label: string, checked = false, disabled = false) =>
  `<label class="ui-choice"${disabled ? ' style="opacity:.5"' : ''}><button class="ui-radio" role="radio" aria-checked="${checked}" tabindex="${checked ? 0 : -1}" id="${id}"${disabled ? ' disabled' : ''} aria-label="${label}"></button>${label}</label>`

const option = (label: string, selected = false, active = false) =>
  `<div class="ui-menu__item${active ? ' is-active' : ''}" role="option" aria-selected="${selected}"><span>${label}</span><span class="ui-menu__shortcut" style="letter-spacing:0">${selected ? ui('check') : ''}</span></div>`

const menuItems = `
  <div class="ui-menu__label">My account</div>
  <div class="ui-menu__sep"></div>
  <div class="ui-menu__item" role="menuitem">${ui('user')}Profile<span class="ui-menu__shortcut">⇧⌘P</span></div>
  <div class="ui-menu__item" role="menuitem">${ui('settings')}Settings<span class="ui-menu__shortcut">⌘,</span></div>
  <div class="ui-menu__item" role="menuitem">${ui('keyboard')}Keyboard shortcuts<span class="ui-menu__shortcut">⌘K</span></div>
  <div class="ui-menu__sep"></div>
  <div class="ui-menu__item" role="menuitemcheckbox" aria-checked="true"><span class="ui-menu__indicator">${ui('check')}</span>Show status bar</div>
  <div class="ui-menu__item" role="menuitem" aria-disabled="true">${ui('mail')}Invite (disabled)</div>
  <div class="ui-menu__sep"></div>
  <div class="ui-menu__item ui-menu__item--destructive" role="menuitem">${ui('log-out')}Log out<span class="ui-menu__shortcut">⇧⌘Q</span></div>`

const treeRow = (depth: number, label: string, icon: string, opts: { open?: boolean; leaf?: boolean; active?: boolean } = {}) =>
  `<div class="ui-menu__item${opts.active ? ' is-active' : ''}" role="treeitem" style="padding-left:${8 + depth * 16}px"${opts.leaf ? '' : ` aria-expanded="${!!opts.open}"`}>
    <span class="ui-muted" style="width:16px;display:inline-grid">${opts.leaf ? '' : ui(opts.open ? 'chevron-down' : 'chevron-right')}</span>${ui(icon)}${label}</div>`

const rows = [
  ['viewport.rs', 'Component', 'Ready', '12.4 KB'],
  ['curve-editor.rs', 'Registry', 'Draft', '31.0 KB'],
  ['histogram.rs', 'Registry', 'Ready', '8.9 KB'],
  ['timeline.rs', 'Registry', 'Planned', '—'],
  ['mkit-core', 'Foundation', 'Ready', '44.2 KB'],
]

export const e7: DemoMap = {
  button: {
    html: `<div class="ui-col" style="gap:16px;align-items:center">
      <div class="ui-row"><button class="ui-btn">Continue</button><button class="ui-btn ui-btn--secondary">Secondary</button><button class="ui-btn ui-btn--outline">Outline</button><button class="ui-btn ui-btn--ghost">Ghost</button><button class="ui-btn ui-btn--destructive">Delete</button><button class="ui-btn ui-btn--link">Link</button></div>
      <div class="ui-row"><button class="ui-btn ui-btn--sm ui-btn--outline">Small</button><button class="ui-btn ui-btn--outline">Default</button><button class="ui-btn ui-btn--lg ui-btn--outline">Large</button><button class="ui-btn" disabled>${ui('loader')}<span>Loading</span></button><button class="ui-btn ui-btn--outline" disabled>Disabled</button></div>
    </div>`,
    mount: (root) => root.querySelector('[disabled] .ui-i-loader')?.classList.add('ui-spin'),
  },
  'icon-button': {
    html: `<div class="ui-row">
      <button class="ui-btn ui-btn--icon" aria-label="Add layer">${ui('plus')}</button>
      <button class="ui-btn ui-btn--icon ui-btn--secondary" aria-label="Duplicate">${ui('copy')}</button>
      <button class="ui-btn ui-btn--icon ui-btn--outline" aria-label="Settings">${ui('settings')}</button>
      <button class="ui-btn ui-btn--icon ui-btn--ghost" aria-label="More">${ui('more-horizontal')}</button>
      <button class="ui-btn ui-btn--icon ui-btn--destructive" aria-label="Delete">${ui('trash')}</button>
      <button class="ui-btn ui-btn--icon ui-btn--outline ui-btn--sm" aria-label="Zoom in">${ui('zoom-in')}</button>
      <button class="ui-btn ui-btn--icon ui-btn--outline" aria-label="Locked" disabled>${ui('lock')}</button>
    </div>`,
  },
  'toggle-button': {
    html: `<div class="ui-row">
      <button class="ui-toggle" data-ui-press aria-pressed="true" aria-label="Bold">${ui('bold')}</button>
      <button class="ui-toggle" data-ui-press aria-pressed="false" aria-label="Italic">${ui('italic')}</button>
      <button class="ui-toggle ui-toggle--outline" data-ui-press aria-pressed="false">${ui('eye')}Preview</button>
      <button class="ui-toggle ui-toggle--outline" aria-pressed="false" disabled style="opacity:.5">${ui('lock')}Locked</button>
    </div>`,
  },
  'toggle-group': {
    html: `<div class="ui-col" style="gap:16px;align-items:center">
      <div class="ui-toggle-group ui-toggle-group--outline" role="radiogroup" aria-label="Alignment">
        <button class="ui-toggle" role="radio" aria-checked="true" aria-label="Align left">${ui('align-left')}</button>
        <button class="ui-toggle" role="radio" aria-checked="false" tabindex="-1" aria-label="Align center">${ui('align-center')}</button>
        <button class="ui-toggle" role="radio" aria-checked="false" tabindex="-1" aria-label="Align right">${ui('align-right')}</button>
      </div>
      <div class="ui-toggle-group" role="radiogroup" aria-label="Weight">
        <button class="ui-toggle" role="radio" aria-checked="false" aria-label="Bold">${ui('bold')}</button>
        <button class="ui-toggle" role="radio" aria-checked="true" aria-label="Italic">${ui('italic')}</button>
        <button class="ui-toggle" role="radio" aria-checked="false" aria-label="Underline">${ui('underline')}</button>
      </div>
    </div>`,
  },
  checkbox: {
    html: `<div class="ui-col" style="gap:14px">
      ${check('Accept terms and conditions', true)}
      <div class="ui-choice-block"><button class="ui-check" role="checkbox" aria-checked="false" aria-label="Enable notifications">${ui('check')}${ui('minus')}</button><div class="ui-col" style="gap:4px"><span class="ui-label">Enable notifications</span><span class="ui-description">You can change this later in settings.</span></div></div>
      ${check('Select all layers', 'mixed')}
      <label class="ui-choice" style="opacity:.5"><button class="ui-check" role="checkbox" aria-checked="false" disabled aria-label="Disabled">${ui('check')}${ui('minus')}</button>Disabled</label>
    </div>`,
  },
  'radio-group': {
    html: `<div class="ui-col" role="radiogroup" aria-label="Density" style="gap:12px">
      ${radio('r-default', 'Default', true)}${radio('r-comfortable', 'Comfortable')}${radio('r-compact', 'Compact')}${radio('r-dense', 'Dense (unavailable)', false, true)}
    </div>`,
  },
  switch: {
    html: `<div class="ui-col" style="gap:16px">
      <label class="ui-choice"><button class="ui-switch" role="switch" aria-checked="true" aria-label="Airplane mode"></button>Airplane mode</label>
      <div class="ui-card" style="display:flex;align-items:center;justify-content:space-between;gap:24px;padding:12px 16px;box-shadow:none">
        <div class="ui-col" style="gap:2px"><span class="ui-label">Snap to grid</span><span class="ui-description">Align objects to the nearest grid line.</span></div>
        <button class="ui-switch" role="switch" aria-checked="false" aria-label="Snap to grid"></button>
      </div>
      <label class="ui-choice" style="opacity:.5"><button class="ui-switch" role="switch" aria-checked="false" disabled aria-label="Sync"></button>Sync (disabled)</label>
    </div>`,
  },
  slider: {
    html: `<div class="ui-col" style="gap:24px;width:min(360px,100%)">
      <div class="ui-field"><div class="ui-row" style="justify-content:space-between"><label class="ui-label" for="s-opacity">Opacity</label><span class="ui-muted ui-small" data-ui-output="s-opacity" data-unit="%">64%</span></div><input class="ui-slider" id="s-opacity" type="range" min="0" max="100" value="64"/></div>
      <div class="ui-field"><div class="ui-row" style="justify-content:space-between"><label class="ui-label" for="s-temp">Temperature</label><span class="ui-muted ui-small" data-ui-output="s-temp">0.4</span></div><input class="ui-slider" id="s-temp" type="range" min="0" max="1" step="0.1" value="0.4"/></div>
      <div class="ui-field" style="opacity:.5"><span class="ui-label">Disabled</span><input class="ui-slider" type="range" value="30" disabled/></div>
    </div>`,
  },
  progress: {
    html: `<div class="ui-col" style="gap:20px;width:min(360px,100%)">
      <div class="ui-field"><div class="ui-row" style="justify-content:space-between"><span class="ui-label">Exporting frames</span><span class="ui-muted ui-small" data-progress-label>66%</span></div><div class="ui-progress" role="progressbar" aria-valuenow="66"><i style="width:66%" data-progress></i></div></div>
      <div class="ui-field"><span class="ui-label">Indexing library</span><div class="ui-progress ui-progress--indeterminate" role="progressbar"><i></i></div></div>
    </div>`,
    mount: (root) => {
      let value = 66
      const bar = root.querySelector<HTMLElement>('[data-progress]')
      const label = root.querySelector<HTMLElement>('[data-progress-label]')
      const timer = window.setInterval(() => {
        value = value >= 100 ? 8 : value + 6
        if (bar) bar.style.width = `${value}%`
        if (label) label.textContent = `${Math.min(value, 100)}%`
      }, 900)
      return () => window.clearInterval(timer)
    },
  },
  'text-field': {
    html: `<div class="ui-col" style="gap:20px;width:min(340px,100%)">
      <div class="ui-field"><label class="ui-label" for="tf-name">Project name</label><input class="ui-input" id="tf-name" value="Laika"/><span class="ui-description">Shown in the window title.</span></div>
      <div class="ui-field"><label class="ui-label" for="tf-email">Email</label><input class="ui-input" id="tf-email" value="not-an-email" aria-invalid="true"/><span class="ui-error-text">Enter a valid email address.</span></div>
      <div class="ui-field"><label class="ui-label" for="tf-off">Workspace</label><input class="ui-input" id="tf-off" placeholder="Read only" disabled/></div>
    </div>`,
  },
  'text-area': {
    html: `<div class="ui-field" style="width:min(380px,100%)"><label class="ui-label" for="ta-notes">Release notes</label><textarea class="ui-textarea" id="ta-notes" rows="4" placeholder="Describe what changed…">Curve editor now snaps keys to whole frames.
Hold Shift for fine adjustment.</textarea><span class="ui-description">Markdown is supported. Undo groups by word.</span></div>`,
  },
  select: {
    html: `<div class="ui-anchor" style="width:220px">
      <button class="ui-input ui-row" style="justify-content:space-between;cursor:pointer;flex-wrap:nowrap" data-ui-open="sel" aria-haspopup="listbox" aria-expanded="true"><span>Active</span><span class="ui-muted">${ui('chevron-down')}</span></button>
      <div class="ui-popover ui-menu ui-float ui-w-full" data-ui-id="sel" data-state="open" role="listbox">
        ${option('Active', true, true)}${option('Paused')}${option('Archived')}<div class="ui-menu__item" role="option" aria-disabled="true">Deleted</div>
      </div>
    </div>`,
    align: 'start',
    minHeight: 280,
  },
  combobox: {
    html: `<div class="ui-anchor" style="width:260px">
      <div class="ui-popover ui-w-full" style="position:static">
        <div class="ui-input-wrap" style="border-bottom:1px solid var(--ui-border)">${ui('search')}<input class="ui-input" style="border:0;box-shadow:none" value="cur" aria-label="Search components"/></div>
        <div class="ui-menu" role="listbox">
          <div class="ui-menu__label ui-muted ui-small" style="font-weight:500">Components</div>
          ${option('<b>Cur</b>ve editor', false, true)}${option('Pre<b>cur</b>sor timeline')}
          <div class="ui-menu__sep"></div>
          <div class="ui-menu__item">${ui('plus')}Use “cur” as a custom value</div>
        </div>
      </div>
    </div>`,
    align: 'start',
  },
  'multi-select': {
    html: `<div style="width:300px" class="ui-col">
      <div class="ui-input ui-row" style="height:auto;min-height:36px;padding:4px 8px;gap:4px;flex-wrap:wrap"><span class="ui-badge ui-badge--secondary">Red ${ui('x')}</span><span class="ui-badge ui-badge--secondary">Blue ${ui('x')}</span><span class="ui-muted" style="margin-left:4px">Add channel…</span><span class="ui-muted" style="margin-left:auto">${ui('chevrons-up-down')}</span></div>
      <div class="ui-popover ui-menu" role="listbox" aria-multiselectable="true">
        <label class="ui-menu__item ui-choice" style="font-weight:400">${box('Red', true)}Red</label>
        <label class="ui-menu__item ui-choice is-active" style="font-weight:400">${box('Green')}Green</label>
        <label class="ui-menu__item ui-choice" style="font-weight:400">${box('Blue', true)}Blue</label>
        <label class="ui-menu__item ui-choice" style="font-weight:400">${box('Alpha')}Alpha</label>
      </div>
    </div>`,
    align: 'start',
  },
  'dropdown-menu': {
    html: `<div class="ui-anchor"><button class="ui-btn ui-btn--outline" data-ui-open="dd" aria-haspopup="menu" aria-expanded="true">Open menu ${ui('chevron-down')}</button>
      <div class="ui-popover ui-menu ui-float" data-ui-id="dd" data-state="open" role="menu" style="width:240px">${menuItems}</div></div>`,
    align: 'start',
    minHeight: 380,
  },
  'context-menu': {
    html: `<div class="ui-col" style="gap:12px;align-items:flex-start">
      <div data-ctx-area style="position:relative;width:min(420px,100%);height:260px;border:1px dashed var(--ui-border);border-radius:var(--ui-radius-lg);display:grid;place-items:center" class="ui-muted">
        <span>Right-click here</span>
        <div class="ui-popover ui-menu" data-ui-id="ctx" data-state="open" role="menu" style="position:absolute;left:40%;top:18px;width:220px">
          <div class="ui-menu__item" role="menuitem">${ui('undo')}Back<span class="ui-menu__shortcut">⌘[</span></div>
          <div class="ui-menu__item" role="menuitem" aria-disabled="true">${ui('chevron-right')}Forward<span class="ui-menu__shortcut">⌘]</span></div>
          <div class="ui-menu__item" role="menuitem">${ui('rotate-ccw')}Reload<span class="ui-menu__shortcut">⌘R</span></div>
          <div class="ui-menu__sep"></div>
          <div class="ui-menu__item" role="menuitemcheckbox" aria-checked="true"><span class="ui-menu__indicator">${ui('check')}</span>Show rulers</div>
          <div class="ui-menu__item" role="menuitemcheckbox" aria-checked="false"><span class="ui-menu__indicator"></span>Show guides</div>
          <div class="ui-menu__sep"></div>
          <div class="ui-menu__item" role="menuitem">More tools<span class="ui-menu__shortcut">${ui('chevron-right')}</span></div>
        </div>
      </div></div>`,
    mount: (root) => {
      const area = root.querySelector<HTMLElement>('[data-ctx-area]')
      const menu = root.querySelector<HTMLElement>('[data-ui-id="ctx"]')
      area?.addEventListener('contextmenu', (event) => {
        event.preventDefault()
        if (!menu) return
        const box = area.getBoundingClientRect()
        menu.style.left = `${Math.min(event.clientX - box.left, box.width - 224)}px`
        menu.style.top = `${Math.min(event.clientY - box.top, box.height - 250)}px`
        menu.dataset.state = 'open'
      })
    },
  },
  tooltip: {
    html: `<div class="ui-row" style="gap:24px;padding-top:40px">
      <span class="ui-anchor"><button class="ui-btn ui-btn--outline">Hover</button><span class="ui-tooltip ui-float ui-float--up" role="tooltip">Add to library</span></span>
      <span class="ui-anchor" data-tip><button class="ui-btn ui-btn--outline ui-btn--icon" aria-label="Zoom to fit">${ui('maximize')}</button><span class="ui-tooltip ui-float ui-float--up" role="tooltip" hidden>Zoom to fit <span style="opacity:.6">⇧1</span></span></span>
    </div>`,
    mount: (root) => {
      root.querySelectorAll<HTMLElement>('[data-tip]').forEach((anchor) => {
        const tip = anchor.querySelector<HTMLElement>('[role="tooltip"]')
        const show = (on: boolean) => tip && (tip.hidden = !on)
        anchor.addEventListener('mouseenter', () => show(true))
        anchor.addEventListener('mouseleave', () => show(false))
        anchor.addEventListener('focusin', () => show(true))
        anchor.addEventListener('focusout', () => show(false))
      })
    },
  },
  popover: {
    html: `<div class="ui-anchor"><button class="ui-btn ui-btn--outline" data-ui-open="pop" aria-expanded="true">Dimensions</button>
      <div class="ui-popover ui-float" data-ui-id="pop" data-state="open" role="dialog" style="width:300px;padding:16px">
        <div class="ui-col" style="gap:4px;margin-bottom:14px"><b style="font-weight:600">Dimensions</b><span class="ui-description">Set the dimensions for the layer.</span></div>
        <div class="ui-grid" style="grid-template-columns:90px 1fr;align-items:center;gap:8px">
          <label class="ui-label" for="p-w">Width</label><input class="ui-input" id="p-w" value="100%" style="height:32px"/>
          <label class="ui-label" for="p-h">Height</label><input class="ui-input" id="p-h" value="25px" style="height:32px"/>
        </div>
      </div></div>`,
    align: 'start',
  },
  dialog: {
    html: `<div class="ui-overlay" data-ui-id="dlg" data-state="open" data-ui-sticky>
      <div class="ui-card" role="dialog" aria-modal="true" aria-labelledby="dlg-t" style="width:min(420px,calc(100% - 32px));padding:24px;box-shadow:var(--ui-shadow-lg);position:relative">
        <button class="ui-btn ui-btn--ghost ui-btn--icon ui-btn--sm" style="position:absolute;right:12px;top:12px;opacity:.7" data-ui-close="dlg" aria-label="Close">${ui('x')}</button>
        <div class="ui-col" style="gap:6px"><h4 id="dlg-t" style="margin:0;font-size:18px;font-weight:600">Discard unsaved changes?</h4><p class="ui-description" style="margin:0;font-size:14px">Your edits to <b>Scene 04</b> will be lost. This can’t be undone.</p></div>
        <div class="ui-row" style="justify-content:flex-end;margin-top:20px"><button class="ui-btn ui-btn--outline" data-ui-close="dlg">Cancel</button><button class="ui-btn ui-btn--destructive" data-ui-close="dlg">Discard</button></div>
      </div></div>
      <button class="ui-btn ui-btn--outline" data-ui-open="dlg">Open dialog</button>`,
    minHeight: 340,
  },
  sheet: {
    html: `<div class="ui-overlay" data-ui-id="sh" data-state="open" data-ui-sticky style="place-items:stretch end">
      <div role="dialog" aria-modal="true" style="width:min(320px,85%);height:100%;background:var(--ui-background);border-left:1px solid var(--ui-border);padding:24px;display:flex;flex-direction:column;gap:18px;position:relative;box-shadow:var(--ui-shadow-lg)">
        <button class="ui-btn ui-btn--ghost ui-btn--icon ui-btn--sm" style="position:absolute;right:12px;top:12px;opacity:.7" data-ui-close="sh" aria-label="Close">${ui('x')}</button>
        <div class="ui-col" style="gap:4px"><b style="font-size:16px;font-weight:600">Edit profile</b><span class="ui-description">Make changes to your profile here.</span></div>
        <div class="ui-field"><label class="ui-label" for="sh-n">Name</label><input class="ui-input" id="sh-n" value="Ada Lovelace"/></div>
        <div class="ui-field"><label class="ui-label" for="sh-u">Username</label><input class="ui-input" id="sh-u" value="@ada"/></div>
        <button class="ui-btn" style="margin-top:auto" data-ui-close="sh">Save changes</button>
      </div></div>
      <button class="ui-btn ui-btn--outline" data-ui-open="sh">Open sheet</button>`,
    minHeight: 380,
  },
  toast: {
    html: `<button class="ui-btn ui-btn--outline" data-toast-trigger>Show toast</button>
      <div class="ui-popover" data-ui-id="toast" data-state="open" data-ui-sticky role="status" style="position:absolute;right:16px;bottom:16px;width:min(340px,calc(100% - 32px));padding:14px 16px;display:flex;gap:12px;align-items:flex-start;box-shadow:var(--ui-shadow-lg);border-radius:var(--ui-radius-lg)">
        <span style="color:var(--ui-success);margin-top:1px">${ui('circle-check')}</span>
        <div class="ui-col" style="gap:2px;flex:1"><b style="font-weight:600;font-size:14px">Export complete</b><span class="ui-description">scene-04.mov · 1.2 GB</span></div>
        <button class="ui-btn ui-btn--outline ui-btn--sm" data-ui-close="toast">Undo</button>
      </div>`,
    mount: (root) => {
      const toast = root.querySelector<HTMLElement>('[data-ui-id="toast"]')
      root.querySelector('[data-toast-trigger]')?.addEventListener('click', () => {
        if (!toast) return
        toast.dataset.state = 'closed'
        requestAnimationFrame(() => (toast.dataset.state = 'open'))
      })
    },
  },
  tabs: {
    html: `<div class="ui-col" style="width:min(400px,100%);gap:8px">
      <div class="ui-tabs__list" role="tablist" aria-label="Settings">
        <button class="ui-tabs__trigger" role="tab" aria-selected="true" aria-controls="tp-account">Account</button>
        <button class="ui-tabs__trigger" role="tab" aria-selected="false" aria-controls="tp-password" tabindex="-1">Password</button>
        <button class="ui-tabs__trigger" role="tab" aria-selected="false" disabled style="opacity:.5">Billing</button>
      </div>
      <div class="ui-card ui-tabs__panel" id="tp-account" role="tabpanel"><div class="ui-card__header"><h4 class="ui-card__title">Account</h4><p class="ui-card__description">Make changes to your account here.</p></div><div class="ui-card__content ui-field"><label class="ui-label" for="tb-n">Name</label><input class="ui-input" id="tb-n" value="Ada Lovelace"/></div></div>
      <div class="ui-card ui-tabs__panel" id="tp-password" role="tabpanel" hidden><div class="ui-card__header"><h4 class="ui-card__title">Password</h4><p class="ui-card__description">Change your password here.</p></div><div class="ui-card__content ui-field"><label class="ui-label" for="tb-p">Current password</label><input class="ui-input" id="tb-p" type="password" value="hunter22"/></div></div>
    </div>`,
  },
  sidebar: {
    html: `<div class="ui-card" style="display:flex;width:min(520px,100%);height:280px;overflow:hidden;box-shadow:none">
      <nav role="radiogroup" aria-label="Workspace" style="width:190px;padding:8px;border-right:1px solid var(--ui-border);background:var(--ui-muted)" class="ui-col">
        <div class="ui-menu__label ui-muted ui-small">Workspace</div>
        ${[['home', 'Home', true], ['inbox', 'Inbox'], ['layers', 'Layers'], ['image', 'Assets'], ['settings', 'Settings']]
          .map(([icon, label, on]) => `<button class="ui-menu__item" role="radio" aria-checked="${!!on}"${on ? '' : ' tabindex="-1"'}>${ui(icon as string)}${label}</button>`)
          .join('')}
      </nav>
      <div style="flex:1;padding:20px" class="ui-col"><div class="ui-skeleton" style="height:18px;width:40%"></div><div class="ui-skeleton" style="height:12px;width:85%"></div><div class="ui-skeleton" style="height:12px;width:70%"></div><div class="ui-skeleton" style="height:90px;margin-top:8px"></div></div>
    </div>`,
  },
  breadcrumbs: {
    html: `<nav aria-label="Breadcrumb"><ol class="ui-row ui-muted" style="list-style:none;margin:0;padding:0;gap:6px;font-size:14px">
      <li><a href="#" onclick="return false" style="color:inherit">Home</a></li><li>${ui('chevron-right', 14)}</li>
      <li><button class="ui-btn ui-btn--ghost ui-btn--sm" style="height:24px;padding:0 4px;color:inherit">${ui('more-horizontal')}</button></li><li>${ui('chevron-right', 14)}</li>
      <li><a href="#" onclick="return false" style="color:inherit">Projects</a></li><li>${ui('chevron-right', 14)}</li>
      <li><span aria-current="page" style="color:var(--ui-foreground)">Scene 04</span></li>
    </ol></nav>`,
  },
  'segmented-control': {
    html: `<div class="ui-col" style="gap:16px;align-items:center">
      <div class="ui-tabs__list" role="radiogroup" aria-label="View">
        <button class="ui-tabs__trigger" role="radio" aria-checked="true">${ui('image')}Canvas</button>
        <button class="ui-tabs__trigger" role="radio" aria-checked="false" tabindex="-1">${ui('layers')}Layers</button>
        <button class="ui-tabs__trigger" role="radio" aria-checked="false" tabindex="-1">${ui('terminal')}Console</button>
      </div>
      <div class="ui-tabs__list" role="radiogroup" aria-label="Units"><button class="ui-tabs__trigger" role="radio" aria-checked="false">px</button><button class="ui-tabs__trigger" role="radio" aria-checked="true">%</button><button class="ui-tabs__trigger" role="radio" aria-checked="false">em</button></div>
    </div>`,
  },
  'virtual-list': {
    html: `<div class="ui-card" style="width:min(340px,100%);box-shadow:none;overflow:hidden">
      <div class="ui-row ui-muted ui-small" style="justify-content:space-between;padding:8px 12px;border-bottom:1px solid var(--ui-border)"><span>10,000 rows</span><span>2 selected</span></div>
      <div role="listbox" aria-multiselectable="true" style="height:224px;overflow:auto;padding:4px" data-vlist></div>
    </div>`,
    mount: (root) => {
      const list = root.querySelector<HTMLElement>('[data-vlist]')
      if (!list) return
      list.innerHTML = Array.from({ length: 60 }, (_, i) =>
        `<div class="ui-menu__item" role="option" aria-selected="${i === 2 || i === 3}" style="height:32px${i === 2 || i === 3 ? ';background:var(--ui-accent)' : ''}"><span class="ui-mono ui-muted" style="width:44px">${String(i + 1).padStart(4, '0')}</span>frame_${String(i * 24).padStart(5, '0')}.exr</div>`,
      ).join('')
    },
  },
  tree: {
    html: `<div role="tree" aria-label="Project" class="ui-card" style="width:min(300px,100%);padding:4px;box-shadow:none">
      ${treeRow(0, 'src', 'folder', { open: true })}
      ${treeRow(1, 'components', 'folder', { open: true })}
      ${treeRow(2, 'viewport.rs', 'file', { leaf: true, active: true })}
      ${treeRow(2, 'curve_editor.rs', 'file', { leaf: true })}
      ${treeRow(1, 'theme', 'folder')}
      ${treeRow(1, 'lib.rs', 'file', { leaf: true })}
      ${treeRow(0, 'assets', 'folder')}
      <div class="ui-menu__item ui-muted" style="padding-left:40px">${ui('loader')}Loading…</div>
    </div>`,
  },
  'data-table': {
    html: `<div class="ui-card" style="width:min(560px,100%);box-shadow:none;overflow:auto">
      <table class="ui-table" role="grid"><thead><tr><th style="width:36px"><button class="ui-check" role="checkbox" aria-checked="mixed" aria-label="Select all">${ui('check')}${ui('minus')}</button></th><th><button class="ui-btn ui-btn--ghost ui-btn--sm" style="margin-left:-8px">Name ${ui('arrow-up-down')}</button></th><th>Type</th><th>Status</th><th style="text-align:right">Size</th></tr></thead>
      <tbody>${rows
        .map(([name, type, status, size], i) => `<tr aria-selected="${i === 1}"><td><button class="ui-check" role="checkbox" aria-checked="${i === 1}" aria-label="Select ${name}">${ui('check')}${ui('minus')}</button></td><td class="ui-mono" style="font-size:13px">${name}</td><td class="ui-muted">${type}</td><td><span class="ui-badge ${status === 'Ready' ? '' : status === 'Draft' ? 'ui-badge--secondary' : 'ui-badge--outline'}">${status}</span></td><td style="text-align:right" class="ui-muted">${size}</td></tr>`)
        .join('')}</tbody></table></div>`,
  },
  'split-pane': {
    html: `<div class="ui-card" data-split style="display:flex;width:min(560px,100%);height:240px;box-shadow:none;overflow:hidden">
      <div data-split-left style="width:38%;padding:16px" class="ui-col"><span class="ui-label">Layers</span><div class="ui-skeleton" style="height:12px"></div><div class="ui-skeleton" style="height:12px;width:75%"></div><div class="ui-skeleton" style="height:12px;width:60%"></div></div>
      <div role="separator" aria-orientation="vertical" aria-valuenow="38" tabindex="0" data-split-handle style="width:1px;background:var(--ui-border);position:relative;cursor:col-resize;display:grid;place-items:center">
        <span style="position:absolute;width:12px;height:18px;border-radius:3px;border:1px solid var(--ui-border);background:var(--ui-border);display:grid;place-items:center">${ui('grip-vertical', 10)}</span></div>
      <div style="flex:1;padding:16px" class="ui-col"><span class="ui-label">Preview</span><div class="ui-skeleton" style="flex:1"></div></div>
    </div>`,
    mount: (root) => {
      const box = root.querySelector<HTMLElement>('[data-split]')
      const left = root.querySelector<HTMLElement>('[data-split-left]')
      const handle = root.querySelector<HTMLElement>('[data-split-handle]')
      if (!box || !left || !handle) return
      const set = (pct: number) => {
        const clamped = Math.max(10, Math.min(90, pct))
        left.style.width = `${clamped}%`
        handle.setAttribute('aria-valuenow', String(Math.round(clamped)))
      }
      handle.addEventListener('pointerdown', (event) => {
        handle.setPointerCapture(event.pointerId)
        const move = (e: PointerEvent) => set(((e.clientX - box.getBoundingClientRect().left) / box.clientWidth) * 100)
        handle.addEventListener('pointermove', move)
        handle.addEventListener('pointerup', () => handle.removeEventListener('pointermove', move), { once: true })
      })
      handle.addEventListener('keydown', (event) => {
        const now = Number(handle.getAttribute('aria-valuenow'))
        if (event.key === 'ArrowLeft') set(now - 5)
        if (event.key === 'ArrowRight') set(now + 5)
      })
    },
  },
  'scroll-area': {
    html: `<div class="ui-card" style="width:220px;height:260px;overflow:auto;box-shadow:none;padding:16px" tabindex="0">
      <b style="font-weight:500;font-size:14px">Tags</b>
      ${Array.from({ length: 30 }, (_, i) => `<div style="font-size:14px;padding:8px 0;border-bottom:1px solid var(--ui-border)">v1.2.0-beta.${30 - i}</div>`).join('')}
    </div>`,
  },
  'form-layout': {
    html: `<div class="ui-card" style="width:min(460px,100%);padding:20px;box-shadow:none" role="group" aria-label="Export settings">
      <div class="ui-grid" style="grid-template-columns:120px 1fr;gap:16px 16px;align-items:start">
        <label class="ui-label" for="fl-n" style="padding-top:11px">File name</label><div class="ui-field"><input class="ui-input" id="fl-n" value="scene-04"/></div>
        <label class="ui-label" style="padding-top:11px">Format</label><div class="ui-field"><button class="ui-input ui-row" style="justify-content:space-between;flex-wrap:nowrap">ProRes 422 <span class="ui-muted">${ui('chevron-down')}</span></button><span class="ui-description">Best for editing. Larger files.</span></div>
        <label class="ui-label" for="fl-f" style="padding-top:11px">Frame rate</label><div class="ui-field"><input class="ui-input" id="fl-f" value="0" aria-invalid="true"/><span class="ui-error-text">Must be between 1 and 240.</span></div>
      </div>
      <div class="ui-row" style="justify-content:flex-end;margin-top:20px"><button class="ui-btn ui-btn--outline">Cancel</button><button class="ui-btn">Export</button></div>
    </div>`,
  },
  separator: {
    html: `<div style="width:min(320px,100%)">
      <div class="ui-col" style="gap:4px"><b style="font-weight:500;font-size:14px">mkit</b><span class="ui-description">An ownable component kit for GPUI.</span></div>
      <hr class="ui-separator" style="margin:16px 0"/>
      <div class="ui-row" style="gap:12px;height:20px;font-size:14px;flex-wrap:nowrap"><span>Book</span><span class="ui-separator ui-separator--v" role="separator" aria-orientation="vertical"></span><span>Components</span><span class="ui-separator ui-separator--v"></span><span>Registry</span></div>
    </div>`,
  },
}
