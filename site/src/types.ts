export type ComponentKind = 'everyday' | 'pro'

export type ComponentStatus = 'ready' | 'in progress' | 'planned'

export type DocImage = {
  src: string
  alt: string
  caption?: string
  /** Display size in CSS pixels (the capture's pixels divided by its scale). */
  width?: number
  height?: number
}

export type DocSection = {
  id: string
  title: string
  html: string
}

export type ComponentDoc = {
  slug: string
  name: string
  /** First paragraph, rendered inline HTML. */
  lede: string
  /** Any remaining preamble blocks, rendered HTML. */
  intro: string
  /** Draft or status notes from blockquotes, rendered inline HTML. */
  notes: string[]
  image?: DocImage
  sections: DocSection[]
  /** Book chapter this came from, relative to book/src. */
  sourcePath: string
  /** Key into the preview demos, from the chapter file name. */
  demoKey: string
  /** Primary registry entry name, or empty when there is no registry entry. */
  registryName: string
  /** Every registry entry this chapter documents (disclosure also covers accordion). */
  registryNames: string[]
  /** Keyboard and accessibility contract from each entry's conformance manifest. */
  contracts: RegistryContract[]
}

export type RegistryContract = {
  name: string
  /** Registry status, verbatim from registry/registry.json. */
  status: string
  /** Spec path relative to the repository root. */
  spec: string
  states: string[]
  keys: { keys: string; when: string; action: string }[]
  role: string
  properties: { name: string; value: string }[]
}

export type CatalogCard = {
  slug: string
  name: string
  kind: ComponentKind
  group: string
  description: string
  status: ComponentStatus
  tags: string[]
  preview: string
  /** Demo key shown live on the catalog card; defaults to the first chapter. */
  featured: string
  accent: string
  components: ComponentDoc[]
}

export type ChapterDoc = {
  slug: string
  title: string
  /** First paragraph, rendered inline HTML. */
  lede: string
  /** Any remaining preamble blocks, rendered HTML. */
  intro: string
  /** Leading blockquote notes, rendered inline HTML. */
  notes: string[]
  sections: DocSection[]
  /** Book chapter this came from, relative to book/src. */
  sourcePath: string
}
