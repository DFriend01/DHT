use std::io::{Result, Error, ErrorKind};
use std::convert::TryFrom;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::net::IpAddr;
use crc::{Crc, CRC_32_CKSUM};
use protobuf::Message;
use rand;

use crate::comm::protogen::api::{UDPMessage, Request, Reply};

const NUM_RAND_BYTES: usize = 2;

pub enum Operation {
    Put = 0,
    Get = 1,
    Delete = 2,
    Wipe = 3,
    Ping = 4,
    Shutdown = 5,
    GetPid = 6,
    GetNearestPrecedingNodeToKey = 7,
    GetSuccessor = 8
}

impl TryFrom<u32> for Operation {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            0 => Ok(Operation::Put),
            1 => Ok(Operation::Get),
            2 => Ok(Operation::Delete),
            3 => Ok(Operation::Wipe),
            4 => Ok(Operation::Ping),
            5 => Ok(Operation::Shutdown),
            6 => Ok(Operation::GetPid),
            7 => Ok(Operation::GetNearestPrecedingNodeToKey),
            8 => Ok(Operation::GetSuccessor),
            _ => Err(Error::new(ErrorKind::InvalidData, "Invalid operation")),
        }
    }
}

pub enum Status {
    Success = 0,
    InvalidKey = 1,
    MissingKey = 2,
    InvalidValue = 3,
    MissingValue = 4,
    KeyNotFound = 5,
    OutOfMemory = 6,
    UndefinedOperation = 7,
    InternalError = 8,
    InvalidValueSize = 9
}

impl TryFrom<u32> for Status {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            0 => Ok(Status::Success),
            1 => Ok(Status::InvalidKey),
            2 => Ok(Status::MissingKey),
            3 => Ok(Status::InvalidValue),
            4 => Ok(Status::MissingValue),
            5 => Ok(Status::KeyNotFound),
            6 => Ok(Status::OutOfMemory),
            7 => Ok(Status::UndefinedOperation),
            8 => Ok(Status::InternalError),
            _ => Err(Error::new(ErrorKind::InvalidData, "Invalid status")),
        }
    }
}

pub fn create_udp_message(message: impl Message, ip: IpAddr, port: u16) -> Result<UDPMessage> {
    let mut udp_message: UDPMessage = UDPMessage::new();
    udp_message.id = generate_message_id(ip, port, NUM_RAND_BYTES)?;
    udp_message.payload = Message::write_to_bytes(&message)?;
    udp_message.checksum = calculate_checksum(&udp_message.id, &udp_message.payload);
    Ok(udp_message)
}

pub fn parse_message(message_bytes: Vec<u8>) -> Result<UDPMessage> {
    Ok(UDPMessage::parse_from_bytes(message_bytes.as_slice())?)
}

pub fn extract_request(udp_message: &UDPMessage) -> Result<Request> {
    Ok(Request::parse_from_bytes(udp_message.payload.as_slice())?)
}

pub fn extract_reply(udp_message: &UDPMessage) -> Result<Reply> {
    Ok(Reply::parse_from_bytes(udp_message.payload.as_slice())?)
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
