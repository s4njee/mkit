import { ui } from '../ui/icons'
import type { DemoMap } from '../ui/types'

// Content colours below (histogram channels, gradient stops, wheel hues,
// clip swatches) are app data shown by the component, not component chrome.

const panel = (title: string, body: string, width = 'min(420px,100%)', extra = '') =>
  `<div class="ui-card" style="width:${width};box-shadow:none;overflow:hidden${extra}">
    <div class="ui-row" style="justify-content:space-between;padding:10px 14px;border-bottom:1px solid var(--ui-border);flex-wrap:nowrap"><span class="ui-label">${title}</span></div>
    ${body}</div>`

const numberField = (label: string, value: string, unit = '') =>
  `<div class="ui-row" style="gap:10px;flex-wrap:nowrap"><span class="ui-muted" style="width:88px;font-size:13px">${label}</span>
    <div class="ui-input ui-row" data-scrub style="height:30px;padding:0 8px;cursor:ew-resize;flex:1;justify-content:space-between;flex-wrap:nowrap;user-select:none" tabindex="0" role="spinbutton" aria-label="${label}" aria-valuenow="${parseFloat(value)}">
      <span class="ui-mono" style="font-size:13px" data-value>${value}</span><span class="ui-muted ui-small">${unit}</span></div></div>`

function wireScrub(root: HTMLElement) {
  root.querySelectorAll<HTMLElement>('[data-scrub]').forEach((field) => {
    const out = field.querySelector<HTMLElement>('[data-value]')!
    const decimals = (out.textContent!.split('.')[1] ?? '').length
    const set = (next: number) => {
      const text = next.toFixed(decimals)
      out.textContent = (next > 0 && out.textContent!.startsWith('+') ? '+' : '') + text
      field.setAttribute('aria-valuenow', text)
    }
    field.addEventListener('pointerdown', (event) => {
      field.setPointerCapture(event.pointerId)
      const start = event.clientX
      const from = parseFloat(out.textContent!)
      const step = decimals ? 1 / 10 ** decimals : 1
      const move = (e: PointerEvent) => set(from + Math.round((e.clientX - start) / (e.shiftKey ? 12 : 3)) * step)
      field.addEventListener('pointermove', move)
      field.addEventListener('pointerup', () => field.removeEventListener('pointermove', move), { once: true })
    })
    field.addEventListener('keydown', (event) => {
      const step = (decimals ? 1 / 10 ** decimals : 1) * (event.shiftKey ? 10 : 1)
      if (event.key === 'ArrowUp' || event.key === 'ArrowRight') set(parseFloat(out.textContent!) + step)
      else if (event.key === 'ArrowDown' || event.key === 'ArrowLeft') set(parseFloat(out.textContent!) - step)
      else return
      event.preventDefault()
    })
  })
}

// Deterministic pseudo-random bins so the histogram reads like image data.
const bins = (seed: number, shift: number) =>
  Array.from({ length: 64 }, (_, i) => {
    const x = i / 63
    const hump = Math.exp(-(((x - shift) / 0.18) ** 2)) + 0.55 * Math.exp(-(((x - 0.82) / 0.08) ** 2))
    return Math.max(0.02, hump * (0.85 + 0.15 * Math.sin(i * seed)))
  })
const area = (values: number[], w: number, h: number) =>
  `M0 ${h} ` + values.map((v, i) => `L${((i / (values.length - 1)) * w).toFixed(1)} ${(h - v * h * 0.85).toFixed(1)}`).join(' ') + ` L${w} ${h} Z`

const wheel = (label: string, dotX: number, dotY: number, size = 84) =>
  `<div class="ui-col" style="align-items:center;gap:6px"><div style="width:${size}px;height:${size}px;border-radius:50%;position:relative;background:radial-gradient(circle,var(--ui-background) 0%,transparent 70%),conic-gradient(from 90deg,#f43f5e,#eab308,#22c55e,#06b6d4,#6366f1,#d946ef,#f43f5e);border:1px solid var(--ui-border)"><i style="position:absolute;left:${dotX}%;top:${dotY}%;width:10px;height:10px;margin:-5px;border-radius:50%;border:2px solid #fff;box-shadow:0 0 0 1px rgba(0,0,0,.4)"></i></div><span class="ui-muted ui-small">${label}</span></div>`

const clip = (left: number, width: number, color: string, label: string, selected = false) =>
  `<div role="gridcell" aria-selected="${selected}" style="position:absolute;top:4px;bottom:4px;left:${left}%;width:${width}%;border-radius:var(--ui-radius-sm);background:color-mix(in srgb,${color} 22%,var(--ui-background));border:1px solid ${selected ? 'var(--ui-foreground)' : `color-mix(in srgb,${color} 55%,transparent)`};font-size:11px;padding:3px 6px;overflow:hidden;white-space:nowrap">${label}</div>`

const timeline = (withPlayhead: boolean) => `<div class="ui-card" style="width:min(620px,100%);box-shadow:none;overflow:hidden">
  ${withPlayhead ? `<div class="ui-row" style="padding:8px 10px;border-bottom:1px solid var(--ui-border);gap:4px"><button class="ui-btn ui-btn--ghost ui-btn--icon ui-btn--sm" aria-label="Go to start">${ui('skip-back')}</button><button class="ui-btn ui-btn--ghost ui-btn--icon ui-btn--sm" aria-label="Play">${ui('play', 14)}</button><span class="ui-mono" style="margin-left:6px">00:00:12.08</span><span class="ui-muted ui-small" style="margin-left:auto">Selected: Title card</span></div>` : ''}
  <div style="display:grid;grid-template-columns:96px 1fr">
    <div style="border-right:1px solid var(--ui-border)"><div style="height:26px;border-bottom:1px solid var(--ui-border)"></div>
      ${['Video', 'Titles', 'Music', 'VO'].map((t, i) => `<div style="height:40px;padding:0 10px;display:flex;align-items:center;font-size:13px;${withPlayhead && i === 1 ? 'background:var(--ui-accent);font-weight:500;' : ''}border-bottom:1px solid var(--ui-border)">${t}</div>`).join('')}</div>
    <div style="position:relative">
      <div class="ui-row ui-muted ui-mono" style="height:26px;border-bottom:1px solid var(--ui-border);justify-content:space-between;padding:0 6px;font-size:10px;flex-wrap:nowrap">${['0s', '5s', '10s', '15s', '20s', '25s', '30s'].map((t) => `<span>${t}</span>`).join('')}</div>
      <div style="position:relative;height:40px;border-bottom:1px solid var(--ui-border)">${clip(0, 42, '#3b82f6', 'A001_C004.mov')}${clip(43, 50, '#3b82f6', 'A002_C011.mov')}</div>
      <div style="position:relative;height:40px;border-bottom:1px solid var(--ui-border)">${clip(30, 22, '#a855f7', 'Title card', withPlayhead)}</div>
      <div style="position:relative;height:40px;border-bottom:1px solid var(--ui-border)">${clip(4, 90, '#22c55e', 'score_v3.wav')}</div>
      <div style="position:relative;height:40px;border-bottom:1px solid var(--ui-border)">${clip(55, 30, '#f59e0b', 'vo_take2.wav')}</div>
      ${withPlayhead ? '<div data-playhead style="position:absolute;top:0;bottom:0;left:40%;width:2px;background:var(--ui-destructive);cursor:ew-resize"><i style="position:absolute;top:0;left:-5px;width:12px;height:10px;background:var(--ui-destructive);clip-path:polygon(0 0,100% 0,50% 100%)"></i></div>' : ''}
    </div>
  </div></div>`

const node = (x: number, y: number, title: string, inputs: string[], outputs: string[], selected = false) =>
  `<div class="ui-card" style="position:absolute;left:${x}px;top:${y}px;width:132px;box-shadow:var(--ui-shadow-md);${selected ? 'outline:2px solid var(--ui-foreground);outline-offset:1px;' : ''}font-size:12px">
    <div style="padding:6px 10px;border-bottom:1px solid var(--ui-border);font-weight:600">${title}</div>
    <div style="padding:6px 0">${inputs.map((p) => `<div style="position:relative;padding:2px 10px"><i style="position:absolute;left:-5px;top:7px;width:9px;height:9px;border-radius:50%;background:var(--ui-background);border:2px solid var(--ui-muted-foreground)"></i>${p}</div>`).join('')}
    ${outputs.map((p) => `<div style="position:relative;padding:2px 10px;text-align:right">${p}<i style="position:absolute;right:-5px;top:7px;width:9px;height:9px;border-radius:50%;background:var(--ui-foreground)"></i></div>`).join('')}</div></div>`

const nodeGraph = (boxSelect: boolean) => `<div style="position:relative;width:min(600px,100%);height:280px;border:1px solid var(--ui-border);border-radius:var(--ui-radius-lg);overflow:hidden;background-color:var(--ui-background);background-image:radial-gradient(var(--ui-border) 1px,transparent 1px);background-size:16px 16px">
  <svg style="position:absolute;inset:0;width:100%;height:100%" fill="none" stroke="var(--ui-muted-foreground)" stroke-width="1.5"><path d="M156 77 C200 77 200 110 236 110"/><path d="M156 197 C200 197 200 132 236 132"/><path d="M372 120 C410 120 410 150 446 150"/></svg>
  ${node(24, 40, 'Image', [], ['RGB'], boxSelect)}${node(24, 160, 'Noise', ['Scale'], ['Fac'], boxSelect)}${node(236, 72, 'Mix', ['A', 'B'], ['Result'], !boxSelect)}${node(446, 112, 'Output', ['Colour'], [])}
  ${boxSelect ? '<div style="position:absolute;left:12px;top:26px;width:160px;height:228px;border:1px dashed var(--ui-foreground);background:color-mix(in srgb,var(--ui-foreground) 6%,transparent);border-radius:4px"></div>' : ''}
  <div class="ui-row" style="position:absolute;right:10px;bottom:10px;gap:4px"><span class="ui-badge ui-badge--outline" style="background:var(--ui-background)">${boxSelect ? '2 selected' : '100%'}</span></div></div>`

export const e8: DemoMap = {
  viewport: {
    html: `<div style="position:relative;width:min(600px,100%);height:300px;border:1px solid var(--ui-border);border-radius:var(--ui-radius-lg);overflow:hidden;background:var(--ui-muted)" data-viewport tabindex="0" aria-label="Canvas viewport">
      <div class="ui-row ui-muted ui-mono" style="position:absolute;left:24px;right:0;top:0;height:20px;background:var(--ui-background);border-bottom:1px solid var(--ui-border);justify-content:space-between;padding:0 8px;font-size:10px;flex-wrap:nowrap"><span>0</span><span>200</span><span>400</span><span>600</span><span>800</span><span>1000</span></div>
      <div style="position:absolute;left:0;top:20px;bottom:0;width:24px;background:var(--ui-background);border-right:1px solid var(--ui-border)"></div>
      <div data-content style="position:absolute;left:50%;top:55%;width:260px;height:170px;margin:-85px 0 0 -130px;border-radius:6px;box-shadow:var(--ui-shadow-lg);background:linear-gradient(135deg,#0ea5e9,#6366f1 45%,#ec4899);transform-origin:center;display:grid;place-items:center;color:#fff;font-weight:600;cursor:grab">Artboard</div>
      <div class="ui-popover ui-row" style="position:absolute;right:10px;bottom:10px;padding:2px;gap:0;flex-wrap:nowrap"><button class="ui-btn ui-btn--ghost ui-btn--icon ui-btn--sm" data-zoom="-1" aria-label="Zoom out">${ui('zoom-out')}</button><span class="ui-mono" data-zoom-label style="width:48px;text-align:center">100%</span><button class="ui-btn ui-btn--ghost ui-btn--icon ui-btn--sm" data-zoom="1" aria-label="Zoom in">${ui('zoom-in')}</button><button class="ui-btn ui-btn--ghost ui-btn--icon ui-btn--sm" data-zoom="0" aria-label="Fit">${ui('maximize')}</button></div>
    </div>`,
    mount: (root) => {
      const content = root.querySelector<HTMLElement>('[data-content]')!
      const label = root.querySelector<HTMLElement>('[data-zoom-label]')!
      let zoom = 1
      let x = 0
      let y = 0
      const apply = () => {
        content.style.transform = `translate(${x}px, ${y}px) scale(${zoom})`
        label.textContent = `${Math.round(zoom * 100)}%`
      }
      root.querySelectorAll<HTMLElement>('[data-zoom]').forEach((b) =>
        b.addEventListener('click', () => {
          const d = Number(b.dataset.zoom)
          if (d === 0) [zoom, x, y] = [1, 0, 0]
          else zoom = Math.min(4, Math.max(0.25, zoom * (d > 0 ? 1.25 : 0.8)))
          apply()
        }),
      )
      content.addEventListener('pointerdown', (event) => {
        content.setPointerCapture(event.pointerId)
        const [sx, sy, ox, oy] = [event.clientX, event.clientY, x, y]
        const move = (e: PointerEvent) => {
          x = ox + e.clientX - sx
          y = oy + e.clientY - sy
          apply()
        }
        content.addEventListener('pointermove', move)
        content.addEventListener('pointerup', () => content.removeEventListener('pointermove', move), { once: true })
      })
    },
  },
  'number-field-pilot': {
    html: panel('Exposure', `<div class="ui-col" style="padding:14px;gap:10px">${numberField('Exposure', '+1.25', 'EV')}${numberField('Gamma', '2.20')}${numberField('Rotation', '45', '°')}<span class="ui-muted ui-small">Drag horizontally to scrub · Shift for fine · ↑↓ to step</span></div>`, 'min(340px,100%)'),
    mount: wireScrub,
  },
  'precision-slider': {
    html: `<div class="ui-col" style="width:min(380px,100%);gap:22px">
      ${['Contrast', 'Balance'].map((name, i) => `<div class="ui-field"><div class="ui-row" style="justify-content:space-between"><label class="ui-label" for="ps-${i}">${name}</label><span class="ui-mono" data-ui-output="ps-${i}">${i ? '0' : '12'}</span></div>
        <div style="position:relative"><i style="position:absolute;left:50%;top:1px;width:1px;height:14px;background:var(--ui-muted-foreground)"></i><input class="ui-slider" id="ps-${i}" type="range" min="-100" max="100" value="${i ? 0 : 12}"/></div>
        <div class="ui-row ui-muted ui-mono" style="justify-content:space-between;font-size:10px"><span>-100</span><span>0</span><span>+100</span></div></div>`).join('')}
      <span class="ui-muted ui-small">Arrow keys step by 1 · Shift steps by 10 · Home/End jump to limits</span></div>`,
  },
  'curve-editor': {
    html: `<div class="ui-card" style="width:min(420px,100%);box-shadow:none;padding:12px">
      <div class="ui-row" style="justify-content:space-between;margin-bottom:10px">
        <div class="ui-tabs__list" role="tablist" style="height:32px"><button class="ui-tabs__trigger" role="tab" aria-selected="true">Master</button><button class="ui-tabs__trigger" role="tab" aria-selected="false" tabindex="-1">Red</button><button class="ui-tabs__trigger" role="tab" aria-selected="false" tabindex="-1">Green</button><button class="ui-tabs__trigger" role="tab" aria-selected="false" tabindex="-1">Blue</button></div>
        <div class="ui-tabs__list" role="radiogroup" style="height:32px"><button class="ui-tabs__trigger" role="radio" aria-checked="false">Linear</button><button class="ui-tabs__trigger" role="radio" aria-checked="true">Smooth</button></div>
      </div>
      <svg data-curve viewBox="0 0 400 240" style="width:100%;height:auto;display:block;border:1px solid var(--ui-border);border-radius:var(--ui-radius-md);touch-action:none">
        <path d="M100 0V240M200 0V240M300 0V240M0 60H400M0 120H400M0 180H400" stroke="var(--ui-border)"/>
        <path d="M0 240 L400 0" stroke="var(--ui-muted-foreground)" stroke-dasharray="3 4"/>
        <path data-line fill="none" stroke="var(--ui-foreground)" stroke-width="2"/>
        <g data-points fill="var(--ui-background)" stroke="var(--ui-foreground)" stroke-width="2"></g>
      </svg>
      <div class="ui-row ui-muted ui-mono" style="justify-content:space-between;margin-top:8px;font-size:11px"><span data-readout>In 0.33 · Out 0.22</span><span>Drag a point</span></div></div>`,
    mount: (root) => {
      const svg = root.querySelector<SVGSVGElement>('[data-curve]')!
      const pts = [[0, 240], [120, 190], [230, 90], [310, 44], [400, 0]]
      const line = svg.querySelector('[data-line]')!
      const group = svg.querySelector('[data-points]')!
      const readout = root.querySelector<HTMLElement>('[data-readout]')!
      const draw = () => {
        // Catmull-Rom through the points, as a cubic Bézier path.
        let d = `M${pts[0][0]} ${pts[0][1]}`
        for (let i = 0; i < pts.length - 1; i++) {
          const [p0, p1, p2, p3] = [pts[i - 1] ?? pts[i], pts[i], pts[i + 1], pts[i + 2] ?? pts[i + 1]]
          d += ` C${p1[0] + (p2[0] - p0[0]) / 6} ${p1[1] + (p2[1] - p0[1]) / 6} ${p2[0] - (p3[0] - p1[0]) / 6} ${p2[1] - (p3[1] - p1[1]) / 6} ${p2[0]} ${p2[1]}`
        }
        line.setAttribute('d', d)
        group.innerHTML = pts.map(([x, y], i) => `<circle cx="${x}" cy="${y}" r="6" data-i="${i}" style="cursor:grab"/>`).join('')
      }
      draw()
      svg.addEventListener('pointerdown', (event) => {
        const i = Number((event.target as Element).getAttribute('data-i'))
        if (Number.isNaN(i) || !(event.target as Element).matches('circle')) return
        svg.setPointerCapture(event.pointerId)
        const move = (e: PointerEvent) => {
          const r = svg.getBoundingClientRect()
          const x = ((e.clientX - r.left) / r.width) * 400
          const y = Math.max(0, Math.min(240, ((e.clientY - r.top) / r.height) * 240))
          const lo = i === 0 ? 0 : pts[i - 1][0] + 8
          const hi = i === pts.length - 1 ? 400 : pts[i + 1][0] - 8
          pts[i] = [i === 0 ? 0 : i === pts.length - 1 ? 400 : Math.max(lo, Math.min(hi, x)), y]
          readout.textContent = `In ${(pts[i][0] / 400).toFixed(2)} · Out ${(1 - y / 240).toFixed(2)}`
          draw()
        }
        svg.addEventListener('pointermove', move)
        svg.addEventListener('pointerup', () => svg.removeEventListener('pointermove', move), { once: true })
      })
    },
  },
  'colour-tools': {
    html: `<div class="ui-card" style="width:min(560px,100%);box-shadow:none;padding:16px;display:grid;grid-template-columns:auto 1fr;gap:20px">
      <div class="ui-col" style="align-items:center">${wheel('Hue · Saturation', 72, 34, 150)}</div>
      <div class="ui-col" style="gap:10px">
        <div class="ui-row" style="flex-wrap:nowrap"><span style="width:36px;height:36px;border-radius:var(--ui-radius-md);background:#38bdf8;border:1px solid var(--ui-border)"></span><input class="ui-input ui-mono" value="#38BDF8" aria-label="Hex"/><button class="ui-btn ui-btn--outline ui-btn--icon" aria-label="Pick colour">${ui('pipette')}</button></div>
        <div class="ui-tabs__list" role="tablist" style="height:30px"><button class="ui-tabs__trigger" role="tab" aria-selected="false">sRGB</button><button class="ui-tabs__trigger" role="tab" aria-selected="true">HSL</button><button class="ui-tabs__trigger" role="tab" aria-selected="false">OKLCH</button></div>
        <div class="ui-grid" style="grid-template-columns:repeat(3,1fr);gap:6px">${[['H', '199°'], ['S', '93%'], ['L', '60%']].map(([k, v]) => `<div class="ui-input ui-row" style="height:30px;padding:0 8px;justify-content:space-between;flex-wrap:nowrap"><span class="ui-muted ui-small">${k}</span><span class="ui-mono">${v}</span></div>`).join('')}</div>
      </div>
      <div class="ui-row" style="grid-column:1/-1;justify-content:space-around;border-top:1px solid var(--ui-border);padding-top:14px">${wheel('Shadows', 40, 62)}${wheel('Midtones', 50, 50)}${wheel('Highlights', 62, 38)}</div>
    </div>`,
  },
  'gradient-editor': {
    html: `<div class="ui-card" style="width:min(460px,100%);box-shadow:none;padding:16px">
      <div style="height:36px;border-radius:var(--ui-radius-md);border:1px solid var(--ui-border);background:linear-gradient(90deg,#0f172a 0%,#6366f1 38%,#f472b6 64%,#fde68a 100%)"></div>
      <div style="position:relative;height:22px;margin:4px 0 14px">${[[0, '#0f172a'], [38, '#6366f1'], [64, '#f472b6', true], [100, '#fde68a']].map(([p, c, on]) => `<button aria-label="Stop at ${p}%" style="position:absolute;left:${p}%;top:0;width:14px;height:18px;margin-left:-7px;padding:0;border-radius:3px 3px 7px 7px;background:${c};border:2px solid ${on ? 'var(--ui-foreground)' : 'var(--ui-background)'};box-shadow:0 0 0 1px var(--ui-border);cursor:pointer"></button>`).join('')}</div>
      <div class="ui-grid" style="grid-template-columns:1fr 1fr auto;gap:8px;align-items:end">
        <div class="ui-field"><span class="ui-label ui-small">Position</span><div class="ui-input ui-row" style="justify-content:space-between;flex-wrap:nowrap"><span class="ui-mono">64</span><span class="ui-muted">%</span></div></div>
        <div class="ui-field"><span class="ui-label ui-small">Colour</span><div class="ui-input ui-row" style="flex-wrap:nowrap"><i style="width:14px;height:14px;border-radius:3px;background:#f472b6"></i><span class="ui-mono">#F472B6</span></div></div>
        <div class="ui-row" style="gap:4px"><button class="ui-btn ui-btn--outline ui-btn--icon" aria-label="Add stop">${ui('plus')}</button><button class="ui-btn ui-btn--outline ui-btn--icon" aria-label="Remove stop">${ui('trash')}</button></div>
      </div></div>`,
  },
  histogram: {
    html: `<div class="ui-card" style="width:min(420px,100%);box-shadow:none;padding:14px">
      <div class="ui-row" style="justify-content:space-between;margin-bottom:10px"><span class="ui-label">Histogram</span><span class="ui-badge ui-badge--outline">RGB</span></div>
      <svg viewBox="0 0 400 150" role="img" aria-label="Luminance skews to the midtones with a highlight peak near white" style="width:100%;height:auto;display:block;border-radius:var(--ui-radius-md);background:var(--ui-muted)">
        <g style="mix-blend-mode:screen" opacity=".7"><path d="${area(bins(1.7, 0.42), 400, 150)}" fill="#ef4444"/><path d="${area(bins(2.3, 0.5), 400, 150)}" fill="#22c55e"/><path d="${area(bins(3.1, 0.36), 400, 150)}" fill="#3b82f6"/></g>
        <path d="${area(bins(1.1, 0.45), 400, 150).replace(/ Z$/, '').replace(/^M0 150 /, 'M')}" fill="none" stroke="var(--ui-foreground)" stroke-width="1.5"/>
      </svg>
      <div class="ui-row ui-muted ui-small" style="justify-content:space-between;margin-top:8px"><span>Shadows</span><span>Midtones</span><span>Highlights</span></div></div>`,
  },
  'property-inspector': {
    html: panel('Transform', `<div class="ui-col" style="padding:12px 14px;gap:10px">
      <div class="ui-row" style="gap:10px;flex-wrap:nowrap"><span class="ui-muted" style="width:88px;font-size:13px">Position</span><div class="ui-grid" style="grid-template-columns:1fr 1fr;gap:6px;flex:1">${['X 24', 'Y 80'].map((v) => `<div class="ui-input ui-row" data-scrub style="height:30px;padding:0 8px;flex-wrap:nowrap;cursor:ew-resize" tabindex="0" role="spinbutton"><span class="ui-muted ui-small">${v[0]}</span><span class="ui-mono" data-value style="margin-left:auto;font-size:13px">${v.slice(2)}</span></div>`).join('')}</div></div>
      ${numberField('Opacity', '86', '%')}
      <div class="ui-row" style="gap:10px;flex-wrap:nowrap"><span class="ui-muted" style="width:88px;font-size:13px">Blend mode</span><button class="ui-input ui-row" style="height:30px;flex:1;justify-content:space-between;flex-wrap:nowrap"><span class="ui-muted" style="font-style:italic">Mixed</span><span class="ui-muted">${ui('chevron-down')}</span></button></div>
      <div class="ui-row" style="gap:10px;flex-wrap:nowrap"><span class="ui-muted" style="width:88px;font-size:13px">Fill</span><div class="ui-input ui-row" style="height:30px;flex:1;flex-wrap:nowrap"><i style="width:14px;height:14px;border-radius:3px;background:#6366f1"></i><span class="ui-mono" style="font-size:13px">#6366F1</span></div></div>
      <div class="ui-row" style="gap:10px;flex-wrap:nowrap"><span class="ui-muted" style="width:88px;font-size:13px">Visible</span><button class="ui-switch" role="switch" aria-checked="true" aria-label="Visible"></button></div>
    </div>`, 'min(360px,100%)'),
    mount: wireScrub,
  },
  'timeline-t1': { html: timeline(false) },
  'timeline-t2': {
    html: timeline(true),
    mount: (root) => {
      const head = root.querySelector<HTMLElement>('[data-playhead]')
      const lane = head?.parentElement
      if (!head || !lane) return
      head.addEventListener('pointerdown', (event) => {
        head.setPointerCapture(event.pointerId)
        const move = (e: PointerEvent) => {
          const r = lane.getBoundingClientRect()
          head.style.left = `${Math.max(0, Math.min(100, ((e.clientX - r.left) / r.width) * 100))}%`
        }
        head.addEventListener('pointermove', move)
        head.addEventListener('pointerup', () => head.removeEventListener('pointermove', move), { once: true })
      })
    },
  },
  'node-editor-n1': { html: nodeGraph(false) },
  'node-editor-n2': { html: nodeGraph(true) },
  'layer-panel': {
    html: panel('Layers', `<div role="tree" style="padding:4px">
      ${[
        [0, 'Artwork', 'linear-gradient(135deg,#6366f1,#ec4899)', true, false, false, true],
        [1, 'Highlights', '#fde68a', false, false, false, false],
        [1, 'Subject', 'linear-gradient(135deg,#0ea5e9,#22c55e)', false, false, true, false],
        [1, 'Shadows', '#1e293b', false, true, false, false],
        [0, 'Background', '#e2e8f0', false, false, false, false],
      ]
        .map(([depth, name, swatch, group, hidden, selected, open]) => `<div class="ui-menu__item${selected ? ' is-active' : ''}" role="treeitem" aria-selected="${selected}" style="padding-left:${6 + (depth as number) * 18}px;${hidden ? 'opacity:.5;' : ''}">
          <span class="ui-muted" style="width:14px;display:inline-grid">${group ? ui(open ? 'chevron-down' : 'chevron-right', 14) : ''}</span>
          <i style="width:22px;height:16px;border-radius:3px;background:${swatch};border:1px solid var(--ui-border)"></i>
          <span style="${selected ? 'font-weight:500' : ''}">${name}</span>
          <span class="ui-row" style="margin-left:auto;gap:2px;flex-wrap:nowrap"><button class="ui-btn ui-btn--ghost ui-btn--icon ui-btn--sm" style="width:24px;height:24px" aria-label="Toggle visibility">${ui(hidden ? 'eye-off' : 'eye', 14)}</button><button class="ui-btn ui-btn--ghost ui-btn--icon ui-btn--sm" style="width:24px;height:24px;${name === 'Background' ? '' : 'opacity:.25'}" aria-label="Toggle lock">${ui('lock', 14)}</button></span></div>`)
        .join('')}</div>`, 'min(320px,100%)'),
  },
  'command-palette': {
    html: `<div class="ui-popover" role="dialog" aria-label="Command palette" style="width:min(460px,100%);box-shadow:var(--ui-shadow-lg);border-radius:var(--ui-radius-lg);overflow:hidden">
      <div class="ui-input-wrap" style="border-bottom:1px solid var(--ui-border)">${ui('search')}<input class="ui-input" data-cmd-input style="border:0;box-shadow:none;height:44px" placeholder="Type a command or search…" aria-label="Search commands"/></div>
      <div class="ui-menu" role="listbox" data-cmd-list style="max-height:250px;overflow:auto">
        ${[
          ['File', [['file', 'Open file…', '⌘O'], ['copy', 'Duplicate layer', '⌘D'], ['image', 'Export frame', '⇧⌘E']]],
          ['View', [['panel-left', 'Toggle sidebar', '⌘B'], ['maximize', 'Zoom to fit', '⇧1'], ['keyboard', 'Keyboard shortcuts', '⌘/']]],
        ]
          .map(([group, items]) => `<div class="ui-menu__label ui-muted ui-small" style="font-weight:500">${group}</div>${(items as string[][]).map(([icon, label, key], i) => `<div class="ui-menu__item${group === 'File' && i === 0 ? ' is-active' : ''}" role="option" data-cmd>${ui(icon)}${label}<span class="ui-menu__shortcut">${key}</span></div>`).join('')}`)
          .join('')}
        <div class="ui-muted" data-cmd-empty hidden style="padding:24px;text-align:center">No results found.</div>
      </div>
      <div class="ui-row ui-muted ui-small" style="border-top:1px solid var(--ui-border);padding:8px 12px;gap:14px"><span><span class="ui-kbd">↑↓</span> navigate</span><span><span class="ui-kbd">↵</span> run</span><span><span class="ui-kbd">esc</span> close</span></div>
    </div>`,
    mount: (root) => {
      const input = root.querySelector<HTMLInputElement>('[data-cmd-input]')!
      const items = [...root.querySelectorAll<HTMLElement>('[data-cmd]')]
      const empty = root.querySelector<HTMLElement>('[data-cmd-empty]')!
      let active = 0
      const visible = () => items.filter((i) => !i.hidden)
      const paint = () => visible().forEach((item, i) => item.classList.toggle('is-active', i === active))
      input.addEventListener('input', () => {
        const q = input.value.toLowerCase()
        items.forEach((item) => (item.hidden = !item.textContent!.toLowerCase().includes(q)))
        root.querySelectorAll<HTMLElement>('[data-cmd-list] .ui-menu__label').forEach((label) => {
          let next = label.nextElementSibling as HTMLElement | null
          let any = false
          while (next && next.hasAttribute('data-cmd')) {
            any ||= !next.hidden
            next = next.nextElementSibling as HTMLElement | null
          }
          label.hidden = !any
        })
        empty.hidden = visible().length > 0
        active = 0
        paint()
      })
      input.addEventListener('keydown', (event) => {
        const count = visible().length
        if (!count) return
        if (event.key === 'ArrowDown') active = (active + 1) % count
        else if (event.key === 'ArrowUp') active = (active - 1 + count) % count
        else return
        event.preventDefault()
        paint()
      })
      items.forEach((item) => item.addEventListener('mousemove', () => {
        active = visible().indexOf(item)
        paint()
      }))
    },
  },
  'shortcut-editor': {
    html: `<div class="ui-card" style="width:min(520px,100%);box-shadow:none;overflow:hidden">
      <div class="ui-row" style="padding:10px;border-bottom:1px solid var(--ui-border);flex-wrap:nowrap"><div class="ui-input-wrap" style="flex:1">${ui('search')}<input class="ui-input" style="height:32px" placeholder="Search actions…" aria-label="Search actions"/></div><button class="ui-btn ui-btn--sm">Save keymap</button></div>
      <table class="ui-table"><thead><tr><th>Action</th><th>Keys</th><th></th></tr></thead><tbody>
        <tr><td>Open command palette</td><td><span class="ui-kbd">⌘</span> <span class="ui-kbd">K</span></td><td></td></tr>
        <tr aria-selected="true"><td style="font-weight:500">Toggle sidebar</td><td><span class="ui-badge ui-badge--outline" style="border-style:dashed">Press keys… <span class="ui-kbd">⌘</span><span class="ui-kbd">B</span></span></td><td class="ui-small" style="color:var(--ui-destructive)">${ui('alert-triangle', 14)} Conflicts with Bold</td></tr>
        <tr><td>Bold</td><td><span class="ui-kbd">⌘</span> <span class="ui-kbd">B</span></td><td></td></tr>
        <tr><td>Focus viewport</td><td><span class="ui-kbd">F6</span></td><td></td></tr>
      </tbody></table></div>`,
  },
}
