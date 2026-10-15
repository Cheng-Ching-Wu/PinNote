import { ref } from 'vue'

import en from './locales/en'
import zhTW, { type Messages } from './locales/zh-TW'

/**
 * UI language. Kept tiny on purpose: two locales, `{name}` placeholders and an
 * English-style plural form (`one | other`, chosen by the `count` param).
 * The Rust side (tray menu, dialogs, error messages) mirrors this in `src-tauri/src/i18n.rs`.
 */
export type Locale = 'zh-TW' | 'en'
/** Stored in settings; `system` follows the OS language. */
export type LanguageSetting = 'system' | Locale

export const LOCALES: readonly Locale[] = ['zh-TW', 'en']
export const LANGUAGE_SETTINGS: readonly LanguageSetting[] = ['system', ...LOCALES]

/** Each language's name in itself, so it can be found whatever the current language is. */
export const LOCALE_NAMES: Record<Locale, string> = {
  'zh-TW': '繁體中文',
  en: 'English',
}

const MESSAGES: Record<Locale, Messages> = { 'zh-TW': zhTW, en }

export function isLanguageSetting(value: unknown): value is LanguageSetting {
  return typeof value === 'string' && (LANGUAGE_SETTINGS as readonly string[]).includes(value)
}

function osLanguages(): readonly string[] {
  if (typeof navigator === 'undefined') return []
  return navigator.languages?.length ? navigator.languages : [navigator.language]
}

/** Any Chinese OS language gets Traditional Chinese; everything else English. Mirrors `i18n::system_locale` in Rust. */
export function systemLocale(languages: readonly string[] = osLanguages()): Locale {
  return languages[0]?.toLowerCase().startsWith('zh') ? 'zh-TW' : 'en'
}

export function resolveLocale(setting: LanguageSetting): Locale {
  return setting === 'system' ? systemLocale() : setting
}

/** Reactive: templates and computeds calling `t` re-render when it changes. */
export const locale = ref<Locale>(systemLocale())

export function setLocale(next: Locale) {
  locale.value = next
  if (typeof document !== 'undefined') document.documentElement.lang = next === 'zh-TW' ? 'zh-Hant' : 'en'
}

/** `{ a: { b: string } }` → `'a.b'` */
type Paths<T> = {
  [K in keyof T & string]: T[K] extends string ? K : `${K}.${Paths<T[K]>}`
}[keyof T & string]

export type MessageKey = Paths<Messages>

type Params = Record<string, string | number>

function lookup(messages: Messages, key: string): string | undefined {
  let node: unknown = messages
  for (const part of key.split('.')) node = (node as Record<string, unknown> | undefined)?.[part]
  return typeof node === 'string' ? node : undefined
}

export function t(key: MessageKey, params?: Params): string {
  let text = lookup(MESSAGES[locale.value], key) ?? lookup(zhTW, key) ?? key
  if (text.includes(' | ')) {
    const [one, other] = text.split(' | ')
    text = params?.count === 1 ? one : other
  }
  return params ? text.replace(/\{(\w+)\}/g, (match, name: string) => String(params[name] ?? match)) : text
}

/** For sorting names: Chinese collation also orders Latin text sensibly. */
export function collatorLocale(): string {
  return locale.value === 'zh-TW' ? 'zh-Hant' : 'en'
}
