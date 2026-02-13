#[allow(dead_code)]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// CRC-32 for DualSense Bluetooth packets
/// Includes phantom header 0xA2 (BT HID Output Report header) processing
pub fn crc32_bt(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    
    // First, process "phantom" BT header 0xA2
    // This byte is not included in the packet payload but is part of CRC calculation
    crc ^= 0xA2u32;
    for _ in 0..8 {
        if (crc & 1) != 0 {
            crc = (crc >> 1) ^ 0xEDB88320;
        } else {
            crc >>= 1;
        }
    }
    
    // Then process the data itself
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}
