#![allow(unreachable_code)]

use std::io::{Result, Error, ErrorKind};
use std::net::SocketAddr;
use std::collections::HashMap;
use log;
use mini_moka::unsync::Cache;
use protobuf::Message;

use crate::comm::ProtoInterface;
use crate::comm::proto::{Operation, Status, extract_request};
use crate::comm::protogen::api::{UDPMessage, Request, Reply};

const MAX_CACHE_SIZE: u64 = 1000;

pub struct Node {
    proto_interface: ProtoInterface,
    data_store: HashMap<Vec<u8>, Vec<u8>>,
    request_cache: Cache<Vec<u8>, Vec<u8>>,
    id: u32,
    max_mem: u32,
}

impl Node {
    pub fn new(socket_addr: SocketAddr, id: u32, max_mem_mb: u32) -> Result<Self> {
        let proto_interface: ProtoInterface = ProtoInterface::new(socket_addr)?;
        let data_store: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let request_cache: Cache<Vec<u8>, Vec<u8>> = Cache::builder()
            .max_capacity(MAX_CACHE_SIZE)
            .time_to_idle(std::time::Duration::from_secs(1))
            .weigher(|k: &Vec<u8>, v: &Vec<u8>| (k.len() + v.len()) as u32)
            .build();

        let max_mem_bytes: u32 = max_mem_mb * 1e6 as u32;
        Ok(Node {proto_interface, data_store, request_cache, id, max_mem: max_mem_bytes})
    }

    pub fn run(&mut self) -> Result<()> {
        log::info!("Server N{} starting up...", self.id);

        loop {
            let (msg, sender_addr) = match self.proto_interface.listen() {
                Ok((msg, addr)) => (msg, addr),
                Err(_) => {
                    continue;
                }
            };

            let reply: Reply = match self.get_reply(msg) {
                Ok(reply) => reply,
                Err(_) => {
                    continue;
                }
            };

            match self.proto_interface.send(reply, sender_addr) {
                Ok(_) => (),
                Err(e) => log::debug!("Failed to send reply: {}", e),
            }
        };

        log::error!("Node run loop exited unexpectedly");
        Err(Error::new(ErrorKind::Other, "Node run loop exited unexpectedly"))
    }

    fn get_reply(&mut self, msg: UDPMessage) -> Result<Reply> {
        let reply: Reply = match self.get_reply_from_cache(&msg) {
            Ok(cached_reply) => cached_reply,
            Err(_) => {
                let reply: Reply = match self.handle_message(&msg) {
                    Ok(reply) => reply,
                    Err(e) => {
                        log::error!("Failed to handle message: {}", e);
                        self.handle_internal_error()
                    }
                };

                match reply.write_to_bytes() {
                    Ok(reply_bytes) => {
                        self.request_cache.insert(msg.id, reply_bytes.to_vec());
                    },
                    Err(e) => {
                        log::error!("Failed to serialize reply: {}", e);
                        return Err(Error::new(ErrorKind::InvalidData, "Failed to serialize reply"));
                    }
                }

                reply
            }
        };

        Ok(reply)
    }

    fn get_reply_from_cache(&mut self, msg: &UDPMessage) -> Result<Reply> {
        match self.request_cache.get(&msg.id) {
            Some(reply_bytes) => {
                log::debug!("Cache hit for message Id of size {}", msg.id.len());
                let reply: Reply = Reply::parse_from_bytes(reply_bytes.as_slice())?;
                Ok(reply)
            },
            None => {
                log::debug!("Cache miss for message Id of size {}", msg.id.len());
                Err(Error::new(ErrorKind::NotFound, "Cache miss"))
            },
        }
    }

    fn handle_message(&mut self, msg: &UDPMessage) -> Result<Reply> {
        let request: Request = extract_request(msg)?;

        if let Ok(Operation::Shutdown) = request.operation.try_into() {
            log::warn!("Received shutdown request, stopping server...");
            std::process::exit(0);
        }

        let reply: Reply = match request.operation.try_into() {
            Ok(Operation::Put) => self.handle_put(request),
            Ok(Operation::Get) => self.handle_get(request),
            Ok(Operation::Delete) => self.handle_delete(request),
            Ok(Operation::Wipe) => self.handle_wipe(),
            Ok(Operation::Ping) => self.handle_ping(),
            _ => self.handle_undefined_operation(request.operation),
        };

        Ok(reply)
    }

    fn handle_put(&mut self, request: Request) -> Reply {

        let mut reply: Reply = Reply::new();

        let key: Vec<u8> = match request.key {
            Some(key) => key,
            None => {
                log::debug!("PUT request MissingKey");
                reply.status = Status::MissingKey as u32;
                return reply;
            }
        };

        let value: Vec<u8> = match request.value {
            Some(value) => value,
            None => {
                log::debug!("PUT request MissingValue");
                reply.status = Status::MissingValue as u32;
                return reply;
            }
        };

        log::debug!("PUT request Success (key size: {}, value size: {})", key.len(), value.len());

        self.data_store.insert(key, value);
        reply.status = Status::Success as u32;
        reply
    }

    fn handle_get(&self, request: Request) -> Reply {
        let mut reply: Reply = Reply::new();

        let key: Vec<u8> = match request.key {
            Some(key) => key,
            None => {
                log::debug!("GET request MissingKey");
                reply.status = Status::MissingKey as u32;
                return reply;
            }
        };

        let value: Vec<u8> = match self.data_store.get(&key) {
            Some(value) => {
                reply.status = Status::Success as u32;
                value.to_vec()
            },
            None => {
                log::debug!("GET request KeyNotFound");
                reply.status = Status::KeyNotFound as u32;
                return reply
            },
        };

        log::debug!("GET request Success (key size: {}, value size: {})", key.len(), value.len());
        reply.value = Some(value);
        reply
    }

    fn handle_delete(&mut self, request: Request) -> Reply {
        let mut reply: Reply = Reply::new();

        let key: Vec<u8> = match request.key {
            Some(key) => key,
            None => {
                log::debug!("DELETE request MissingKey");
                reply.status = Status::MissingKey as u32;
                return reply;
            }
        };

        match self.data_store.remove(&key) {
            Some(value) => {
                reply.status = Status::Success as u32;
                reply.value = Some(value);
            },
            None => reply.status = {
                log::debug!("DELETE request KeyNotFound");
                Status::KeyNotFound as u32
            },
        };

        log::debug!("DELETE request Success (key size: {})", key.len());

        reply
    }

    fn handle_wipe(&mut self) -> Reply {
        let mut reply: Reply = Reply::new();
        self.data_store = HashMap::new();
        reply.status = Status::Success as u32;

        log::debug!("WIPE request Success");

        reply
    }

    fn handle_ping(&self) -> Reply {
        let mut reply: Reply = Reply::new();
        reply.status = Status::Success as u32;
        log::debug!("PING request Success");
        reply
    }

    fn handle_undefined_operation(&self, bad_error_code: u32) -> Reply {
        let mut reply: Reply = Reply::new();
        reply.status = Status::UndefinedOperation as u32;
        log::debug!("Undefined operation with code {}", bad_error_code);
        reply
    }

    fn handle_internal_error(&self) -> Reply {
        let mut reply: Reply = Reply::new();
        reply.status = Status::InternalError as u32;
        log::debug!("Internal error");
        reply
    }
}
