export type ComponentKind = 'everyday' | 'pro'

export type ComponentStatus = 'ready' | 'in progress' | 'planned'

export type DocImage = {
  src: string
  alt: string
  caption?: string
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
  /** `cargo mkit add` name, or empty when there is no registry entry. */
  registryName: string
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
