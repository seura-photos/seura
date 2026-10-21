//! The language and region the app works in (ACC-5 to ACC-7).
//!
//! - The interface is English in v1.
//! - Dates, numbers, and sizes follow the operating system's region, even
//!   in English (ACC-6). The web view's own locale follows the display
//!   language instead, so the core reports the region and the UI formats
//!   with it.
//! - `SEURA_LOCALE=pseudo` pseudo-translates every string (ACC-7).

/// The developer switch for pseudo-translation.
pub const LOCALE_SWITCH: &str = "SEURA_LOCALE";

pub fn pseudo() -> bool {
    std::env::var(LOCALE_SWITCH).as_deref() == Ok("pseudo")
}

/// Text as a translation might run: about 40% longer, and accented, so
/// clipped or untranslated text stands out (ACC-7). Each vowel is doubled
/// and common letters accented; the UI does the same (`ui/src/pseudo.ts`).
#[cfg_attr(
    not(any(target_os = "macos", test)),
    expect(dead_code, reason = "only the macOS menu uses the core's text so far")
)]
pub fn pseudo_translate(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        let accented = accent(c);
        out.push(accented);
        if "aeiouAEIOU".contains(c) {
            out.push(accented);
        }
    }
    out
}

fn accent(c: char) -> char {
    const FROM: &str = "aceinouyACEINOUY";
    const TO: &str = "áçéîñöûýÁÇÉÎÑÖÛÝ";
    FROM.chars()
        .position(|from| from == c)
        .and_then(|i| TO.chars().nth(i))
        .unwrap_or(c)
}

/// The operating system's region, as a BCP 47 tag such as `en-US` or
/// `de-DE`, for formatting dates and numbers (ACC-6).
pub fn region() -> String {
    platform_region()
        .and_then(|identifier| to_bcp47(&identifier))
        .unwrap_or_else(|| "en-US".into())
}

/// Turns an identifier such as `de-DE`, `en_DE`, or macOS's
/// `en_US@rg=dezzzz` (English, with Germany's formats) into a BCP 47 tag.
fn to_bcp47(identifier: &str) -> Option<String> {
    let (base, keywords) = identifier.split_once('@').unwrap_or((identifier, ""));
    let mut parts: Vec<String> = base
        .split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect();
    let override_region = keywords
        .split(';')
        .find_map(|keyword| keyword.strip_prefix("rg="))
        .filter(|rg| rg.len() >= 2 && rg.is_char_boundary(2))
        .map(|rg| rg[..2].to_uppercase());
    if let Some(region) = override_region {
        let is_region =
            |part: &String| part.len() == 2 && part.chars().all(|c| c.is_ascii_uppercase());
        match parts.iter().position(is_region) {
            Some(i) => parts[i] = region,
            None => parts.push(region),
        }
    }
    let language = parts.first()?;
    (language.len() >= 2 && language.chars().all(|c| c.is_ascii_alphabetic()))
        .then(|| parts.join("-"))
}

#[cfg(target_os = "windows")]
fn platform_region() -> Option<String> {
    // The "Regional format" in Windows' settings.
    let mut name = [0u16; 85];
    #[allow(
        unsafe_code,
        reason = "a Win32 call into a buffer of the documented size"
    )]
    let length = unsafe { windows::Win32::Globalization::GetUserDefaultLocaleName(&mut name) };
    (length > 1).then(|| String::from_utf16_lossy(&name[..length as usize - 1]))
}

#[cfg(target_os = "macos")]
fn platform_region() -> Option<String> {
    let locale = objc2_core_foundation::CFLocale::current()?;
    Some(locale.identifier()?.to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn platform_region() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_become_bcp47_tags() {
        assert_eq!(to_bcp47("de-DE").as_deref(), Some("de-DE"));
        assert_eq!(to_bcp47("en_DE").as_deref(), Some("en-DE"));
        assert_eq!(to_bcp47("en_US@rg=dezzzz").as_deref(), Some("en-DE"));
        assert_eq!(to_bcp47("zh_Hans_CN").as_deref(), Some("zh-Hans-CN"));
        assert_eq!(to_bcp47("").as_deref(), None);
    }

    #[test]
    fn the_region_is_a_tag() {
        let region = region();
        assert!(region.len() >= 2, "{region}");
        assert!(!region.contains('_'), "{region}");
    }

    #[test]
    fn pseudo_translation_is_longer_and_accented() {
        let english = "Choose a folder on the left.";
        let pseudo = pseudo_translate(english);
        assert!(
            pseudo.chars().count() as f64 >= english.len() as f64 * 1.3,
            "{pseudo}"
        );
        assert!(!pseudo.contains('o'), "{pseudo}");
        assert!(pseudo.contains('ö'), "{pseudo}");
    }
}
