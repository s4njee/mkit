import type { DemoMap } from '../ui/types'
import { e7 } from './e7'
import { e7Expansion } from './e7_expansion'
import { e8 } from './e8'
import './e7_expansion.css'

/** Live previews keyed by book chapter file name (see `ComponentDoc.demoKey`). */
export const demos: DemoMap = { ...e7, ...e7Expansion, ...e8 }
