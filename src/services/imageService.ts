import { convertFileSrc, invoke } from '@tauri-apps/api/core'

/** Notes reference their own images as `images/<file>` (relative, portable Markdown). */
const LOCAL_PREFIX = 'images/'

const EXTENSION_BY_MIME: Record<string, string> = {
  'image/png': 'png',
  'image/jpeg': 'jpg',
  'image/gif': 'gif',
  'image/webp': 'webp',
  'image/bmp': 'bmp',
}

export const IMAGE_MIME_TYPES = Object.keys(EXTENSION_BY_MIME)

let imagesDir: string | null = null

export const imageService = {
  /** Must run before any note renders so `resolveSrc` can work synchronously. */
  async init() {
    try {
      imagesDir = await invoke<string>('get_images_dir')
    } catch (error) {
      console.error('[imageService] cannot resolve image folder', error)
    }
  },

  /** `images/<file>` → asset-protocol URL; web URLs and anything else stay unchanged. */
  resolveSrc(src: string | null): string | null {
    if (!src || !imagesDir || !src.startsWith(LOCAL_PREFIX)) return src
    const file = src.slice(LOCAL_PREFIX.length)
    if (!file || /[\\/]|\.\./.test(file)) return src
    return convertFileSrc(`${imagesDir}\\${file}`)
  },

  /** Deletes images no note uses (files from the last 24 h are always kept). */
  cleanupUnused(): Promise<{ count: number; bytes: number }> {
    return invoke('cleanup_images')
  },

  /** Saves a pasted / dropped / picked image; resolves to its Markdown path. */
  async save(file: File): Promise<string> {
    const extension = EXTENSION_BY_MIME[file.type] ?? file.name.split('.').pop() ?? 'png'
    const bytes = new Uint8Array(await file.arrayBuffer())
    return invoke<string>('save_image', bytes, { headers: { 'x-image-ext': extension } })
  },
}
