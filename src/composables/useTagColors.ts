import { resolveTagColor, TAG_COLOR_VALUES } from '@/constants/tagColors'
import { useSettingsStore } from '@/stores/settings'

/** Dot color of a tag, following the user's choice in settings (synced across windows). */
export function useTagColors() {
  const settings = useSettingsStore()
  return {
    tagColor: (name: string) => resolveTagColor(name, settings.settings.tagColors),
    tagColorValue: (name: string) => TAG_COLOR_VALUES[resolveTagColor(name, settings.settings.tagColors)],
  }
}
