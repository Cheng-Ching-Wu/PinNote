/** User preferences stored in the SQLite `settings` table. */
export interface AppSettings {
  /** Pinned (always-on-top) notes turn translucent while not focused / hovered. */
  pinnedTranslucent: boolean
}

export const DEFAULT_SETTINGS: AppSettings = {
  pinnedTranslucent: true,
}
