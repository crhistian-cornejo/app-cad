/// Convert RGB color to object ID for picking
pub fn color_to_id(color: [u8; 3]) -> u32 {
    (color[0] as u32) | ((color[1] as u32) << 8) | ((color[2] as u32) << 16)
}

/// Convert object ID to RGB color for picking
pub fn id_to_color(id: u32) -> [u8; 3] {
    [
        (id & 0xFF) as u8,
        ((id >> 8) & 0xFF) as u8,
        ((id >> 16) & 0xFF) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_id_conversion() {
        let id = 12345u32;
        let color = id_to_color(id);
        let back = color_to_id(color);
        assert_eq!(id, back);
    }
}