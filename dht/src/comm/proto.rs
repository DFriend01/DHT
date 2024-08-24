use crc::{Crc, CRC_32_CKSUM};
use crate::comm::protogen::api::UDPMessage;

pub fn calculate_checksum(id: &[u8], payload: &[u8]) -> u64 {
    let mut msg_content: Vec<u8> = Vec::new();
    msg_content.extend_from_slice(id);
    msg_content.extend_from_slice(payload);

    let crc32 = Crc::<u32>::new(&CRC_32_CKSUM);
    let mut digest = crc32.digest();
    digest.update(&msg_content);
    digest.finalize() as u64
}

pub fn validate_checksum(message: &UDPMessage) -> std::io::Result<()> {
    let checksum: u64 = message.checksum;
    let recalc_checksum: u64 = calculate_checksum(&message.id, &message.payload);
    if checksum == recalc_checksum {
        Ok(())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Checksum mismatch"))
    }
}
