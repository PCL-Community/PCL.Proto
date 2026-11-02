//! Offline UUID derivation, mirroring `ProfileUi.GetOfflineUuid`.

/// 标准离线 UUID：`OfflinePlayer:<名称>` 的 MD5，按 RFC 4122 v3 打上版本与变体位。
pub fn offline(user_name: &str) -> String {
    let mut hash = md5::compute(format!("OfflinePlayer:{user_name}")).0;
    hash[6] = (hash[6] & 0x0f) | 0x30;
    hash[8] = (hash[8] & 0x3f) | 0x80;
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// PCL 旧版离线 UUID（CE 的 `isLegacy` 分支），由名称长度与 PCL 哈希拼成。
pub fn legacy(user_name: &str) -> String {
    let length = fill(&format!("{:X}", user_name.chars().count()), '0', 16);
    let hash = fill(
        &format!("{:X}", crate::utils::secret::get_pcl_hash(user_name)),
        '0',
        16,
    );
    let full = format!("{length}{hash}");
    // CE 直接把版本位与变体位写成 '3' 和 '9'。
    let chars: Vec<char> = full.chars().collect();
    let mut value: String = chars[..12].iter().collect();
    value.push('3');
    value.push_str(&chars[13..16].iter().collect::<String>());
    value.push('9');
    value.push_str(&chars[17..32].iter().collect::<String>());
    value
}

/// 去掉连字符并校验自定义 UUID，对照 CE 的 `^[a-fA-F0-9]{32}$` 检查。
pub fn normalize_custom(value: &str) -> Option<String> {
    let value: String = value.chars().filter(|c| *c != '-').collect();
    (value.len() == 32 && value.chars().all(|c| c.is_ascii_hexdigit())).then_some(value)
}

/// 没有自定义皮肤时按 UUID 选择默认皮肤，对照 CE 的 `ModSkin.McSkinSex`。
pub fn default_skin(uuid: &str) -> &'static str {
    let digits: Vec<char> = uuid.chars().collect();
    if digits.len() != 32 {
        return "Steve";
    }
    let hex = |index: usize| digits[index].to_digit(16).unwrap_or(0);
    if (hex(7) ^ hex(15) ^ hex(23) ^ hex(31)) % 2 != 0 {
        "Alex"
    } else {
        "Steve"
    }
}

/// CE 的 `StrFill`：超长截断，不足左侧补字符。
fn fill(value: &str, code: char, length: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() > length {
        return chars[..length].iter().collect();
    }
    std::iter::repeat_n(code, length - chars.len())
        .chain(chars)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_uuids_match_ce() {
        assert_eq!(offline("Player"), "a01e3843e5213998958af459800e4d11");
        assert_eq!(offline("Steve"), "5627dd98e6be3c21b8a8e92344183641");
        assert_eq!(legacy("Player"), "0000000000003006998F555B7570E899");
        assert_eq!(legacy("amagicpear"), "000000000000300A9D827A125618C12C");
    }

    #[test]
    fn custom_uuids_keep_their_case_and_reject_junk() {
        assert_eq!(
            normalize_custom("A01E3843-E521-3998-958A-F459800E4D11").as_deref(),
            Some("A01E3843E5213998958AF459800E4D11")
        );
        assert_eq!(normalize_custom("tooshort"), None);
        assert_eq!(normalize_custom(&"g".repeat(32)), None);
    }

    #[test]
    fn default_skin_follows_uuid_parity() {
        assert_eq!(default_skin(&offline("Player")), "Alex");
        assert_eq!(default_skin(&offline("Alex")), "Steve");
        assert_eq!(default_skin("short"), "Steve");
    }
}
