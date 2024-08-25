use std::io::{Result, Error, ErrorKind};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::net::IpAddr;
use crc::{Crc, CRC_32_CKSUM};
use protobuf::Message;
use rand;

use crate::comm::protogen::api::UDPMessage;

const NUM_RAND_BYTES: usize = 2;

pub fn create_udp_message(message: impl Message, ip: IpAddr, port: u16) -> Result<UDPMessage> {
    let mut udp_message: UDPMessage = UDPMessage::new();
    udp_message.id = generate_message_id(ip, port, NUM_RAND_BYTES)?;
    udp_message.payload = Message::write_to_bytes(&message)?;
    udp_message.checksum = calculate_checksum(&udp_message.id, &udp_message.payload);
    Ok(udp_message)
}

pub fn calculate_checksum(id: &[u8], payload: &[u8]) -> u64 {
    let mut msg_content: Vec<u8> = Vec::new();
    msg_content.extend_from_slice(id);
    msg_content.extend_from_slice(payload);

    let crc32 = Crc::<u32>::new(&CRC_32_CKSUM);
    let mut digest = crc32.digest();
    digest.update(&msg_content);
    digest.finalize() as u64
}

pub fn validate_checksum(message: &UDPMessage) -> Result<()> {
    let checksum: u64 = message.checksum;
    let recalc_checksum: u64 = calculate_checksum(&message.id, &message.payload);
    if checksum == recalc_checksum {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::InvalidData, "Checksum mismatch"))
    }
}

fn generate_message_id(ip: IpAddr, port: u16, num_rand_bytes: usize) -> Result<Vec<u8>> {
    let binding: String = ip.to_string();
    let ip_bytes: &[u8] = binding.as_bytes();
    let port_bytes: [u8; 2] = port.to_be_bytes();

    let nano_sec_dur: Duration = match SystemTime::now().duration_since(UNIX_EPOCH){
        Ok(n) => n,
        Err(e) => return Err(Error::new(ErrorKind::InvalidData, e)),
    };
    let nano_sec_bytes: [u8; 16] = nano_sec_dur.as_nanos().to_be_bytes();

    let mut rand_bytes: Vec<u8> = Vec::new();
    for _ in 0..num_rand_bytes {
        rand_bytes.push(rand::random::<u8>());
    }

    let mut id: Vec<u8> = Vec::new();
    id.extend_from_slice(&ip_bytes);
    id.extend_from_slice(&port_bytes);
    id.extend_from_slice(&nano_sec_bytes);
    id.extend_from_slice(&rand_bytes);

    Ok(id)
}
