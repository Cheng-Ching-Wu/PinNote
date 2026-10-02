import { describe, expect, it } from 'vitest'

import { formatAccelerator, recordAccelerator } from '@/utils/accelerator'

const key = (code: string, mods: Partial<Record<'ctrlKey' | 'altKey' | 'shiftKey' | 'metaKey', boolean>> = {}) => ({
  code,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  metaKey: false,
  ...mods,
})

describe('recordAccelerator', () => {
  it('builds modifier combinations in a stable order', () => {
    expect(recordAccelerator(key('KeyN', { altKey: true, ctrlKey: true }))).toEqual({
      kind: 'accelerator',
      accelerator: 'Ctrl+Alt+N',
    })
    expect(recordAccelerator(key('Digit1', { metaKey: true, shiftKey: true }))).toEqual({
      kind: 'accelerator',
      accelerator: 'Shift+Super+1',
    })
    expect(recordAccelerator(key('ArrowUp', { ctrlKey: true }))).toEqual({
      kind: 'accelerator',
      accelerator: 'Ctrl+ArrowUp',
    })
  })

  it('accepts bare function keys such as keyboard macro keys', () => {
    expect(recordAccelerator(key('F15'))).toEqual({ kind: 'accelerator', accelerator: 'F15' })
    expect(recordAccelerator(key('F24'))).toEqual({ kind: 'accelerator', accelerator: 'F24' })
  })

  it('waits while only modifiers are held', () => {
    expect(recordAccelerator(key('ControlLeft', { ctrlKey: true, altKey: true }))).toEqual({
      kind: 'pending',
      preview: 'Ctrl + Alt',
    })
  })

  it('rejects bare typing keys', () => {
    expect(recordAccelerator(key('KeyN')).kind).toBe('invalid')
    expect(recordAccelerator(key('Space')).kind).toBe('invalid')
  })
})

describe('formatAccelerator', () => {
  it('shows friendly names', () => {
    expect(formatAccelerator('Ctrl+Super+ArrowUp')).toBe('Ctrl + Win + ↑')
    expect(formatAccelerator('F15')).toBe('F15')
    expect(formatAccelerator('')).toBe('未設定')
  })
})
