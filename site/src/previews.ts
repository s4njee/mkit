import { icon } from './icons'

const bars = '<span></span><span></span><span></span><span></span><span></span><span></span><span></span><span></span>'
const node = (label: string, cls = '') => `<div class="node ${cls}"><i></i><b>${label}</b><em></em></div>`

const previews: Record<string, string> = {
  buttons: `<div class="preview-row"><button class="mini-button solid">Save changes</button><button class="mini-button outline">Cancel</button><button class="mini-icon">•••</button></div><div class="micro-line"><i></i><span>Publish when ready</span><strong>⌘ ↵</strong></div>`,
  controls: `<div class="control-stack"><div class="fake-check"><i>✓</i><span>Use system theme</span></div><div class="range"><span style="width:64%"></span><b></b></div><div class="micro-line"><span>Opacity</span><strong>64%</strong></div></div>`,
  fields: `<div class="field-stack"><div class="fake-input active">Search components <span>⌘ K</span></div><div class="fake-input error">Invalid value <small>Required</small></div></div>`,
  select: `<div class="fake-input">Select a component <span>⌄</span></div><div class="option-list"><b>Viewport</b><span>Curve editor</span><span>Histogram</span></div>`,
  menu: `<div class="menu-list"><span>New file <kbd>⌘ N</kbd></span><span class="selected">Duplicate <kbd>⌘ D</kbd></span><span>View options <b>›</b></span><span class="muted">Archive</span></div>`,
  overlays: `<div class="overlay-demo"><div class="fake-dialog"><b>Publish changes</b><span>Your changes are ready to ship.</span><div><button class="mini-button solid">Publish</button><button class="mini-button outline">Cancel</button></div></div><i class="toast">Changes saved</i></div>`,
  navigation: `<div class="nav-demo"><div class="side-rail"><b>m</b><span class="on"></span><span></span><span></span></div><div class="nav-main"><div class="tabs"><b>Overview</b><span>Activity</span><span>Settings</span></div><div class="fake-copy"></div><div class="fake-copy short"></div></div></div>`,
  table: `<div class="table-demo"><div class="table-head"><b>Name</b><b>Type</b><b>Status</b></div><div><span>viewport.rs</span><small>Component</small><i>Ready</i></div><div><span>curve-editor</span><small>Registry</small><em>Draft</em></div><div><span>mkit-core</span><small>Foundation</small><i>Ready</i></div></div>`,
  layout: `<div class="split-demo"><div class="split-left"><span></span><span></span><span></span></div><i></i><div class="split-right"><b>Inspector</b><span></span><span></span></div></div>`,
  viewport: `<div class="viewport-demo"><div class="ruler-x">0&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; 50&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; 100</div><div class="ruler-y">0<br/><br/>50<br/><br/>100</div><div class="canvas-art"><div></div><b></b><i></i></div><span class="zoom-chip">100%</span></div>`,
  number: `<div class="number-demo"><span>Exposure</span><div><b>+1.25</b><small>EV</small></div><i>↔ drag to scrub</i></div>`,
  precision: `<div class="precision-demo"><div class="precision-label"><span>Contrast</span><b>+12</b></div><div class="bipolar"><i></i><span></span><b></b></div><small>Shift for fine adjustment</small></div>`,
  curve: `<div class="curve-demo"><div class="curve-grid"><svg viewBox="0 0 260 130" preserveAspectRatio="none"><path d="M0 118 C48 118 62 87 103 74 S175 48 260 12"/><circle cx="103" cy="74" r="4"/><circle cx="175" cy="48" r="4"/></svg></div><div class="curve-tabs"><b>Master</b><span>Red</span><span>Green</span><span>Blue</span></div></div>`,
  colour: `<div class="colour-demo"><div class="colour-wheel"><i></i></div><div class="colour-fields"><span>HSL <b>184°</b></span><span>HEX <b>#6EE7F9</b></span><span>Lightness <b>62%</b></span></div></div>`,
  gradient: `<div class="gradient-demo"><div class="gradient-strip"></div><div class="stops"><i></i><i></i><i></i></div><div class="gradient-fields"><span>Position <b>64%</b></span><span>Colour <b>#B48CFF</b></span></div></div>`,
  histogram: `<div class="histogram-demo"><div class="hist-label"><span>Luminance</span><b>RGB</b></div><div class="histogram-bars">${bars}</div><div class="hist-line"></div></div>`,
  inspector: `<div class="inspector-demo"><div class="inspector-head"><b>Transform</b><span>⌃</span></div><div><span>Position</span><b>24, 80</b></div><div><span>Opacity</span><b>86%</b></div><div class="mixed"><span>Blend mode</span><em>Mixed</em></div></div>`,
  timeline: `<div class="timeline-demo"><div class="time-head"><span>00:00</span><span>00:15</span><span>00:30</span><span>00:45</span></div><div><b>Video</b><i style="width:62%"></i></div><div><b>Audio</b><i class="purple" style="width:38%"></i></div><div><b>Titles</b><i class="pink" style="width:23%;margin-left:44%"></i></div><strong></strong></div>`,
  nodes: `<div class="nodes-demo">${node('Input', 'node-a')}${node('Colour', 'node-b')}${node('Output', 'node-c')}<svg viewBox="0 0 300 135"><path d="M70 46 C105 46 108 80 140 80 S185 42 217 42"/><path d="M70 104 C106 104 108 80 140 80"/></svg></div>`,
  layers: `<div class="layers-demo"><div><span>⌄</span><b class="thumb"></b><strong>Artwork</strong><i>◉</i></div><div class="indent"><span></span><b class="thumb pink"></b><strong>Highlights</strong><i>◉</i></div><div class="indent selected"><span></span><b class="thumb blue"></b><strong>Background</strong><i>◉</i></div></div>`,
  palette: `<div class="palette-demo"><div class="palette-search">${icon('search', 13)}<span>Type a command...</span><kbd>ESC</kbd></div><div class="palette-group">File</div><div class="palette-row active"><b>Open file</b><kbd>⌘ O</kbd></div><div class="palette-row"><b>Save file</b><kbd>⌘ S</kbd></div></div>`,
  shortcuts: `<div class="shortcuts-demo"><div><span>Open command palette</span><kbd>⌘ K</kbd></div><div class="active"><span>Toggle sidebar</span><kbd>⌘ B</kbd></div><div><span>Focus viewport</span><kbd>F6</kbd></div></div>`,
  laika: `<div class="laika-demo"><div class="laika-orb"></div><div class="laika-bars"><span></span><span></span><span></span></div><small>Seeded from a real app</small></div>`,
}

export function miniPreview(type: string) {
  return previews[type] ?? ''
}
