/**
 * Converts keyboard events to accelerator strings understood by the Rust side
 * (`tauri-plugin-global-shortcut`), e.g. "Ctrl+Alt+N", "Shift+F5", "F15".
 * Validation mirrors `parse_accelerator` in `src-tauri/src/services/shortcut.rs`.
 */

import { t } from '@/i18n'

export type RecordResult =
  | { kind: 'accelerator'; accelerator: string }
  /** Only modifiers are held so far — keep waiting; `preview` shows them. */
  | { kind: 'pending'; preview: string }
  | { kind: 'invalid'; reason: string }

const MODIFIER_CODES = new Set([
  'ControlLeft',
  'ControlRight',
  'AltLeft',
  'AltRight',
  'ShiftLeft',
  'ShiftRight',
  'MetaLeft',
  'MetaRight',
])

const FUNCTION_KEY = /^F([1-9]|1\d|2[0-4])$/

type KeyLike = Pick<KeyboardEvent, 'code' | 'ctrlKey' | 'altKey' | 'shiftKey' | 'metaKey'>

function modifiers(event: KeyLike): string[] {
  const mods: string[] = []
  if (event.ctrlKey) mods.push('Ctrl')
  if (event.altKey) mods.push('Alt')
  if (event.shiftKey) mods.push('Shift')
  if (event.metaKey) mods.push('Super')
  return mods
}

/** KeyboardEvent.code → key token ("KeyN" → "N", "Digit1" → "1", others unchanged). */
function keyToken(code: string): string {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3)
  if (/^Digit\d$/.test(code)) return code.slice(5)
  return code
}

export function recordAccelerator(event: KeyLike): RecordResult {
  const mods = modifiers(event)

  if (!event.code || MODIFIER_CODES.has(event.code)) {
    return { kind: 'pending', preview: mods.map(displayToken).join(' + ') }
  }

  const key = keyToken(event.code)
  if (mods.length === 0 && !FUNCTION_KEY.test(key)) {
    return { kind: 'invalid', reason: t('shortcuts.invalidSingleKey') }
  }
  return { kind: 'accelerator', accelerator: [...mods, key].join('+') }
}

function displayToken(token: string): string {
  switch (token) {
    case 'Super':
      return 'Win'
    case 'ArrowUp':
      return '↑'
    case 'ArrowDown':
      return '↓'
    case 'ArrowLeft':
      return '←'
    case 'ArrowRight':
      return '→'
    default:
      return token
  }
}

/** "Ctrl+Super+ArrowUp" → "Ctrl + Win + ↑"; "" → "未設定" / "Not set". */
export function formatAccelerator(accelerator: string): string {
  if (!accelerator) return t('common.notSet')
  return accelerator.split('+').map(displayToken).join(' + ')
}
