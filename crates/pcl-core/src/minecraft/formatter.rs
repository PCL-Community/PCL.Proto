#![allow(dead_code)]
//! Minecraft version formatting and wiki URLs, mirroring
//! `PCL.Core.Minecraft.McFormatter`.

/// Format an internal version id as the name used by the Minecraft Wiki.
pub fn format_version(game_version: &str) -> String {
    let id = game_version.to_ascii_lowercase();
    let mapped = match id.as_str() {
        "0.30-1" | "0.30-2" | "c0.30_01c" => Some("Classic_0.30"),
        "in-20100206-2103" => Some("Indev_20100206"),
        "inf-20100630-1" => Some("Infdev_20100630"),
        "inf-20100630-2" => Some("Alpha_v1.0.0"),
        "1.19_deep_dark_experimental_snapshot-1" => Some("1.19-exp1"),
        "in-20100130" => Some("Indev_0.31_20100130"),
        "b1.6-tb3" => Some("Beta_1.6_Test_Build_3"),
        "1_14_combat-212796" => Some("1.14.3_-_Combat_Test"),
        "1_14_combat-0" => Some("Combat_Test_2"),
        "1_14_combat-3" => Some("Combat_Test_3"),
        "1_15_combat-1" => Some("Combat_Test_4"),
        "1_15_combat-6" => Some("Combat_Test_5"),
        "1_16_combat-0" => Some("Combat_Test_6"),
        "1_16_combat-1" => Some("Combat_Test_7"),
        "1_16_combat-2" => Some("Combat_Test_7b"),
        "1_16_combat-3" => Some("Combat_Test_7c"),
        "1_16_combat-4" => Some("Combat_Test_8"),
        "1_16_combat-5" => Some("Combat_Test_8b"),
        "1_16_combat-6" => Some("Combat_Test_8c"),
        _ => None,
    };
    if let Some(mapped) = mapped {
        return mapped.into();
    }
    if id.starts_with("1.0.0-rc2") {
        return "RC2".into();
    }
    if id.starts_with("2.0") || id.starts_with("2point0") {
        return "2.0".into();
    }
    if id.starts_with("b1.8-pre1") {
        return "Beta_1.8-pre1".into();
    }
    if id.starts_with("b1.1-") {
        return "Beta_1.1".into();
    }
    if id.starts_with("a1.1.0") {
        return "Alpha_v1.1.0".into();
    }
    if id.starts_with("a1.0.14") {
        return "Alpha_v1.0.14".into();
    }
    if id.starts_with("a1.0.13_01") {
        return "Alpha_v1.0.13_01".into();
    }
    if id.starts_with("in-20100214") {
        return "Indev_20100214".into();
    }
    if id.contains("experimental-snapshot") {
        return id.replace("_experimental-snapshot-", "-exp");
    }
    if let Some(tail) = id.strip_prefix("inf-") {
        return format!("Infdev_{tail}");
    }
    if let Some(tail) = id.strip_prefix("in-") {
        return format!("Indev_{tail}");
    }
    if id.starts_with("rd-") {
        return format!("pre-Classic_{id}");
    }
    if let Some(tail) = id.strip_prefix('b') {
        return format!("Beta_{tail}");
    }
    if let Some(tail) = id.strip_prefix('a') {
        return format!("Alpha_v{tail}");
    }
    if let Some(tail) = id.strip_prefix('c') {
        return format!("Classic_{tail}").replace("st", "SURVIVAL_TEST");
    }
    id
}

/// Wiki language prefix used by CE for the current UI language.
pub fn wiki_prefix(language: &str) -> Option<&'static str> {
    const PREFIXES: [&str; 13] = [
        "de", "es", "fr", "it", "ja", "ko", "lzh", "nl", "pt", "ru", "th", "uk", "zh",
    ];
    PREFIXES
        .into_iter()
        .find(|prefix| language.to_ascii_lowercase().starts_with(prefix))
}

pub fn wiki_base_url(language: &str) -> String {
    match wiki_prefix(language) {
        Some(prefix) => format!("https://{prefix}.minecraft.wiki"),
        None => "https://minecraft.wiki".into(),
    }
}

/// Build the path part of a wiki link for a game version.
pub fn wiki_url_suffix(game_version: &str, language: &str) -> String {
    let formatted = format_version(game_version);
    if !language.to_ascii_lowercase().starts_with("zh") {
        return format!(
            "/w/Special:Search?search={}",
            urlencoding(&format!("Java Edition {formatted}"))
        );
    }
    if game_version.contains('w') {
        return formatted;
    }
    format!("Java版{formatted}")
}

fn urlencoding(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            b' ' => vec!['+'],
            other => format!("%{other:02X}").chars().collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_legacy_snapshots_and_combat_tests() {
        assert_eq!(format_version("1.20.1"), "1.20.1");
        assert_eq!(format_version("b1.7.3"), "Beta_1.7.3");
        assert_eq!(format_version("a1.2.6"), "Alpha_v1.2.6");
        assert_eq!(format_version("inf-20100630-1"), "Infdev_20100630");
        assert_eq!(format_version("1_16_combat-6"), "Combat_Test_8c");
        assert_eq!(
            format_version("1.19_deep_dark_experimental_snapshot-1"),
            "1.19-exp1"
        );
        assert_eq!(format_version("c0.30_01c"), "Classic_0.30");
    }

    #[test]
    fn builds_language_specific_wiki_links() {
        assert_eq!(wiki_base_url("zh-CN"), "https://zh.minecraft.wiki");
        assert_eq!(wiki_base_url("de-DE"), "https://de.minecraft.wiki");
        assert_eq!(wiki_base_url("en-US"), "https://minecraft.wiki");
        assert_eq!(wiki_url_suffix("1.20.1", "zh-CN"), "Java版1.20.1");
        assert_eq!(
            wiki_url_suffix("1.20.1", "en-US"),
            "/w/Special:Search?search=Java+Edition+1.20.1"
        );
    }
}
