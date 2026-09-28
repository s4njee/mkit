import { catalog } from './generated/components'
import type { CatalogCard, ComponentKind, ComponentStatus } from './types'

export const allCards: CatalogCard[] = catalog
export const everydayCards = catalog.filter((card) => card.kind === 'everyday')
export const proCards = catalog.filter((card) => card.kind === 'pro')
export const cardBySlug = new Map(catalog.map((card) => [card.slug, card]))

/** Distinct registry entries documented by a set of cards. */
export const registryCount = (cards: CatalogCard[]) =>
  new Set(cards.flatMap((card) => card.components.flatMap((doc) => doc.registryNames))).size

export function cardsFor(activeFilter: 'all' | ComponentKind) {
  return activeFilter === 'all' ? allCards : catalog.filter((card) => card.kind === activeFilter)
}

export function previousNext(slug: string) {
  const index = catalog.findIndex((card) => card.slug === slug)
  return {
    previous: index > 0 ? catalog[index - 1] : undefined,
    next: index >= 0 && index < catalog.length - 1 ? catalog[index + 1] : undefined,
  }
}

export const statusClass = (status: string) => status.replace(' ', '-')

// Catalog status describes the documentation, not a release: every registry
// entry is still `implementation_in_progress`.
const statusLabels: Record<ComponentStatus, string> = {
  ready: 'Documented',
  'in progress': 'In progress',
  planned: 'Planned',
}
export const statusLabel = (status: ComponentStatus) => statusLabels[status] ?? status

export const kindLabel = (kind: ComponentKind) => (kind === 'pro' ? 'Pro-app' : 'Everyday')

/** Plain-language registry status, e.g. `implementation_in_progress` → "implementation in progress". */
export const registryStatusLabel = (status: string) => status.replace(/_/g, ' ')

export const accentVar = (accent: string) => `var(--${accent}, var(--coral))`
