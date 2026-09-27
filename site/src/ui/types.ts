/**
 * A live web preview of an mkit component, drawn with the shadcn theme tokens
 * from mkit-core. These are illustrations of the GPUI component for the site,
 * not the component itself; the harness screenshot stays the evidence.
 */
export type Demo = {
  /** Markup rendered inside the preview canvas. Use `.ui-*` primitives. */
  html: string
  /**
   * Optional wiring for demo-specific behaviour. Shared behaviours (checkbox,
   * switch, radio, tabs, toggles, popovers, ranges) are already handled by
   * `mountBehaviors`. Return a cleanup function if you add global listeners.
   */
  mount?: (root: HTMLElement) => void | (() => void)
  /** Canvas alignment. Defaults to `center`. */
  align?: 'center' | 'start' | 'stretch'
  /** Minimum canvas height in px. Defaults to 320. */
  minHeight?: number
}

export type DemoMap = Record<string, Demo>

/** Small static previews for the home page catalog cards, keyed by `preview`. */
export type CardPreviewMap = Record<string, string>
