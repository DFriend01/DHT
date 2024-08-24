use crc::{Crc, CRC_32_CKSUM};
use crate::comm::protogen::api::UDPMessage;

pub fn calculate_checksum(message: &UDPMessage) -> u64 {
    let mut msg_content: Vec<u8> = Vec::new();
    msg_content.extend_from_slice(&message.id);
    msg_content.extend_from_slice(&message.payload);

    let crc32 = Crc::<u32>::new(&CRC_32_CKSUM);
    let mut digest = crc32.digest();
    digest.update(&msg_content);
    digest.finalize() as u64
}

pub fn is_checksum_valid(message: &UDPMessage) -> bool {
    let checksum: u64 = message.checksum;
    let recalc_checksum: u64 = calculate_checksum(message);
    checksum == recalc_checksum
}
