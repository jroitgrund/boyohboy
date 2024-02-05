pub fn get_lsb(val: u8) -> u8 {
    val & (!val + 1)
}

pub fn clear_bit(val: u8, bit: u8) -> u8 {
    return val & !(1 << bit);
}

pub fn set_bit(val: u8, bit: u8) -> u8 {
    return val | (1 << bit);
}

pub fn get_bits(instruction: u8, high_bit: u8, low_bit: u8) -> u8 {
    return (instruction >> low_bit) & ((1 << (1 + high_bit - low_bit)) - 1);
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_get_msb() {
        assert_eq!(super::get_lsb(0b00110010), 0b00000010);
    }

    #[test]
    fn test_clear_bit() {
        assert_eq!(super::clear_bit(0b00110010, 4), 0b00100010);
    }

    #[test]
    fn test_get_bits() {
        assert_eq!(super::get_bits(0b00110010, 5, 1), 0b11001);
    }
}
