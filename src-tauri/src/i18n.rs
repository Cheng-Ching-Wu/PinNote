//! UI language of native text: tray menu, file dialogs and error messages shown
//! by the frontend. The webviews translate themselves (`src/i18n`); both follow
//! the `language` setting ("system" | "zh-TW" | "en").

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    ZhTw,
    En,
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn current() -> Lang {
    match CURRENT.load(Ordering::Relaxed) {
        1 => Lang::En,
        _ => Lang::ZhTw,
    }
}

/// Applies a `language` setting value. Unknown values follow the OS, like "system".
pub fn apply(setting: &str) {
    let lang = match setting {
        "zh-TW" => Lang::ZhTw,
        "en" => Lang::En,
        _ => system_lang(),
    };
    CURRENT.store(lang as u8, Ordering::Relaxed);
}

/// Any Chinese OS language gets Traditional Chinese; everything else English.
/// Mirrors `systemLocale` in `src/i18n/index.ts`.
fn system_lang() -> Lang {
    lang_for_locale(sys_locale::get_locale().as_deref())
}

fn lang_for_locale(locale: Option<&str>) -> Lang {
    match locale {
        Some(locale) if locale.to_lowercase().starts_with("zh") => Lang::ZhTw,
        _ => Lang::En,
    }
}

/// Picks the text for the current language.
pub fn pick(zh: &'static str, en: &'static str) -> &'static str {
    match current() {
        Lang::ZhTw => zh,
        Lang::En => en,
    }
}

/// `format!` with one template per language:
/// `tr!("無法讀取 {name}", "Could not read {name}")`.
#[macro_export]
macro_rules! tr {
    ($zh:literal, $en:literal $(, $arg:expr)* $(,)?) => {
        match $crate::i18n::current() {
            $crate::i18n::Lang::ZhTw => format!($zh $(, $arg)*),
            $crate::i18n::Lang::En => format!($en $(, $arg)*),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chinese_locales_get_traditional_chinese() {
        assert_eq!(lang_for_locale(Some("zh-TW")), Lang::ZhTw);
        assert_eq!(lang_for_locale(Some("zh-Hans-CN")), Lang::ZhTw);
        assert_eq!(lang_for_locale(Some("en-US")), Lang::En);
        assert_eq!(lang_for_locale(Some("ja-JP")), Lang::En);
        assert_eq!(lang_for_locale(None), Lang::En);
    }
}
