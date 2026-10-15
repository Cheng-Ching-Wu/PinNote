import { t } from '@/i18n'

const pad = (value: number) => String(value).padStart(2, '0')

function startOfDay(date: Date) {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime()
}

/** 今天 08:32 / 昨天 17:20 / 2026/09/30 14:05 (Today 08:32… in English) */
export function formatNoteTime(iso: string, now = new Date()): string {
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return ''

  const time = `${pad(date.getHours())}:${pad(date.getMinutes())}`
  const dayDiff = Math.round((startOfDay(now) - startOfDay(date)) / 86_400_000)

  if (dayDiff === 0) return t('time.today', { time })
  if (dayDiff === 1) return t('time.yesterday', { time })
  return `${date.getFullYear()}/${pad(date.getMonth() + 1)}/${pad(date.getDate())} ${time}`
}
