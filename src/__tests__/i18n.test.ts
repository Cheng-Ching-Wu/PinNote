import { afterEach, describe, expect, it } from 'vitest'

import { setLocale, systemLocale, t } from '@/i18n'
import en from '@/i18n/locales/en'
import zhTW from '@/i18n/locales/zh-TW'

/** Every `{name}` placeholder in a message. */
const placeholders = (text: string) => [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort()

function flatten(node: object, prefix = ''): Record<string, string> {
  return Object.entries(node).reduce<Record<string, string>>((acc, [key, value]) => {
    const path = prefix ? `${prefix}.${key}` : key
    return typeof value === 'string' ? { ...acc, [path]: value } : { ...acc, ...flatten(value, path) }
  }, {})
}

describe('i18n', () => {
  afterEach(() => setLocale('zh-TW'))

  it('has the same placeholders in every language', () => {
    const zh = flatten(zhTW)
    const english = flatten(en)
    expect(Object.keys(english).sort()).toEqual(Object.keys(zh).sort())
    for (const [key, text] of Object.entries(zh)) {
      for (const form of english[key].split(' | ')) {
        expect(new Set(placeholders(form)), key).toEqual(new Set(placeholders(text)))
      }
    }
  })

  it('fills placeholders and picks the plural form', () => {
    setLocale('zh-TW')
    expect(t('main.noteCount', { count: 1 })).toBe('1 張便籤')
    setLocale('en')
    expect(t('main.noteCount', { count: 1 })).toBe('1 note')
    expect(t('main.noteCount', { count: 3 })).toBe('3 notes')
    expect(t('tags.delete', { name: 'Work' })).toBe('Delete tag "Work"')
  })

  it('maps any Chinese OS language to Traditional Chinese', () => {
    expect(systemLocale(['zh-TW'])).toBe('zh-TW')
    expect(systemLocale(['zh-CN', 'en'])).toBe('zh-TW')
    expect(systemLocale(['en-US', 'zh-TW'])).toBe('en')
    expect(systemLocale(['ja-JP'])).toBe('en')
    expect(systemLocale([])).toBe('en')
  })
})
