// Shared interactions for preview demos, wired by delegation on the canvas.
// They mirror the documented keyboard contracts closely enough to explore a
// demo; the GPUI component's own tests remain the source of truth.

function setRangeFill(input: HTMLInputElement) {
  const min = Number(input.min || 0)
  const max = Number(input.max || 100)
  const pct = ((Number(input.value) - min) / (max - min)) * 100
  input.style.setProperty('--pct', `${pct}%`)
  const root = input.closest('.ui')
  const output = input.id ? root?.querySelector<HTMLElement>(`[data-ui-output="${input.id}"]`) : null
  if (output) output.textContent = `${input.value}${output.dataset.unit ?? ''}`
}

function selectInGroup(item: HTMLElement, groupSelector: string, itemSelector: string, attr: 'aria-checked' | 'aria-selected') {
  const group = item.closest(groupSelector)
  if (!group) return
  group.querySelectorAll<HTMLElement>(itemSelector).forEach((other) => {
    const on = other === item
    other.setAttribute(attr, String(on))
    if (other.getAttribute('role') === 'tab' || other.getAttribute('role') === 'radio') other.tabIndex = on ? 0 : -1
    const panelId = other.getAttribute('aria-controls')
    const panel = panelId ? group.closest('.ui')?.querySelector<HTMLElement>(`#${panelId}`) : null
    if (panel) panel.hidden = !on
  })
}

function closeAll(root: HTMLElement, except?: Element | null) {
  root.querySelectorAll<HTMLElement>('[data-ui-id][data-state="open"]').forEach((el) => {
    if (el === except || el.dataset.uiSticky !== undefined) return
    el.dataset.state = 'closed'
    root.querySelector(`[data-ui-open="${el.dataset.uiId}"]`)?.setAttribute('aria-expanded', 'false')
  })
}

export function mountBehaviors(root: HTMLElement) {
  root.querySelectorAll<HTMLInputElement>('input[type="range"]').forEach(setRangeFill)

  root.addEventListener('input', (event) => {
    const target = event.target as HTMLElement
    if (target instanceof HTMLInputElement && target.type === 'range') setRangeFill(target)
  })

  root.addEventListener('click', (event) => {
    const target = event.target as HTMLElement
    const opener = target.closest<HTMLElement>('[data-ui-open]')
    const closer = target.closest<HTMLElement>('[data-ui-close]')
    if (closer) {
      const panel = root.querySelector<HTMLElement>(`[data-ui-id="${closer.dataset.uiClose}"]`)
      if (panel) panel.dataset.state = 'closed'
      return
    }
    if (opener) {
      const panel = root.querySelector<HTMLElement>(`[data-ui-id="${opener.dataset.uiOpen}"]`)
      if (panel) {
        const open = panel.dataset.state !== 'open'
        closeAll(root, panel)
        panel.dataset.state = open ? 'open' : 'closed'
        opener.setAttribute('aria-expanded', String(open))
      }
      return
    }
    if (!target.closest('[data-ui-id]')) closeAll(root)

    const control = target.closest<HTMLElement>('[role="checkbox"], [role="switch"], [role="menuitemcheckbox"], [data-ui-press]')
    if (control && !control.hasAttribute('disabled') && control.getAttribute('aria-disabled') !== 'true') {
      if (control.hasAttribute('data-ui-press')) {
        control.setAttribute('aria-pressed', String(control.getAttribute('aria-pressed') !== 'true'))
      } else {
        control.setAttribute('aria-checked', String(control.getAttribute('aria-checked') !== 'true'))
      }
      return
    }
    // A <label class="ui-choice"> wrapping a control forwards clicks to it.
    const label = target.closest<HTMLElement>('.ui-choice')
    const inner = label?.querySelector<HTMLElement>('[role="checkbox"], [role="switch"], [role="radio"]')
    if (label && inner && !inner.contains(target)) {
      inner.click()
      return
    }
    const radio = target.closest<HTMLElement>('[role="radio"]')
    if (radio && !radio.hasAttribute('disabled')) {
      selectInGroup(radio, '[role="radiogroup"]', '[role="radio"]', 'aria-checked')
      return
    }
    const tab = target.closest<HTMLElement>('[role="tab"]')
    if (tab) selectInGroup(tab, '[role="tablist"]', '[role="tab"]', 'aria-selected')
  })

  root.addEventListener('keydown', (event) => {
    const target = event.target as HTMLElement
    if (event.key === 'Escape') {
      closeAll(root)
      return
    }
    const role = target.getAttribute('role')
    if (role !== 'tab' && role !== 'radio') return
    const group = target.closest(role === 'tab' ? '[role="tablist"]' : '[role="radiogroup"]')
    if (!group) return
    const items = [...group.querySelectorAll<HTMLElement>(`[role="${role}"]:not([disabled])`)]
    const index = items.indexOf(target)
    const delta = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[event.key]
    let next: HTMLElement | undefined
    if (delta) next = items[(index + delta + items.length) % items.length]
    if (event.key === 'Home') next = items[0]
    if (event.key === 'End') next = items[items.length - 1]
    if (!next) return
    event.preventDefault()
    next.focus()
    next.click()
  })
}
