import { catalog } from './generated/components'
import type { CatalogCard, ComponentKind } from './types'

export const allCards: CatalogCard[] = catalog
export const everydayCards = catalog.filter((card) => card.kind === 'everyday')
export const proCards = catalog.filter((card) => card.kind === 'pro')
export const cardBySlug = new Map(catalog.map((card) => [card.slug, card]))

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

export const accentVar = (accent: string) => `var(--${accent}, var(--coral))`
