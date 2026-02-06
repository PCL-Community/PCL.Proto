#[cfg(test)]
mod tests {
    use super::*;
    use terracotta::controller::*;

    #[test]
    fn test_parse_code() {
        let codes = [
            "U/LX2M-2A87-YXMZ-2HJJ",
            "U/YS3D-LTH4-6AUC-MBFB",
            "U/BC4D-A51Z-ZE3P-LAP9",
            "U/UCAW-UY61-QD93-UL9X",
        ];
        for code in codes {
            let room = Room::from(code);
            assert!(room.is_some());
        }
    }
}
