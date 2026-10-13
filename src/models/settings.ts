/** User preferences stored in the SQLite `settings` table. */
export interface AppSettings {
  /** Pinned (always-on-top) notes turn translucent while not focused / hovered. */
  pinnedTranslucent: boolean
  /** Tag key (`tagKey`) → color from `TAG_COLORS`. Tags without an entry get an automatic color. */
  tagColors: Record<string, string>
  /** Days a note stays in the trash before it is deleted for good; 0 = never. */
  trashRetentionDays: number
  /** Sync folder on this device; '' = sync off. */
  syncFolder: string
}

export const DEFAULT_SETTINGS: AppSettings = {
  pinnedTranslucent: true,
  tagColors: {},
  trashRetentionDays: 30,
  syncFolder: '',
}
