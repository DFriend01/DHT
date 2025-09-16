#![allow(unreachable_code)]

use std::io::{Result, Error, ErrorKind};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::process;
use log;
use mini_moka::unsync::Cache;
use protobuf::Message;

use crate::comm::ProtoInterface;
use crate::comm::proto::{Operation, Status, extract_request};
use crate::comm::protogen::api::{UDPMessage, Request, Reply};

const MAX_CACHE_CAPACITY_PERCENT: f64 = 0.1;
const MAX_VALUE_PAYLOAD_SIZE_BYTES: usize = 1024 * 10;

pub struct Node {
    proto_interface: ProtoInterface,
    data_store: HashMap<Vec<u8>, Vec<u8>>,
    request_cache: Cache<Vec<u8>, Vec<u8>>,
    id: u32,
    max_mem: u64,
    process_id: u32,
    data_store_mem_usage: u64,
    should_keep_running: bool,
}

impl Node {
    pub fn new(socket_addr: SocketAddr, id: u32, max_mem_mb: u32) -> Result<Self> {
        let proto_interface: ProtoInterface = ProtoInterface::new(socket_addr)?;
        let data_store: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let max_mem_bytes: u64 = (max_mem_mb as u64) * 1024 * 1024;
        let process_id: u32 = process::id();
        let request_cache: Cache<Vec<u8>, Vec<u8>> = Cache::builder()
            .max_capacity((MAX_CACHE_CAPACITY_PERCENT * max_mem_bytes as f64) as u64)
            .time_to_idle(std::time::Duration::from_secs(1))
            .weigher(|k: &Vec<u8>, v: &Vec<u8>| (k.len() + v.len()) as u32)
            .build();

        Ok(Node {
            proto_interface,
            data_store,
            request_cache,
            id,
            max_mem: max_mem_bytes,
            process_id: process_id,
            data_store_mem_usage: 0,
            should_keep_running: true,
        } )
    }

    pub fn run(&mut self) -> Result<()> {
        log::info!("Server N{} starting up...", self.id);

        while self.should_keep_running {
            let (msg, sender_addr) = match self.proto_interface.listen() {
                Ok((msg, addr)) => {
                    log::trace!("Received message from {}", addr);
                    (msg, addr)
                },
                Err(e) => {
                    log::trace!("Failed to receive message: {}", e);
                    continue;
                }
            };

            let reply: Reply = match self.get_reply(msg) {
                Ok(reply) => reply,
                Err(e) => {
                    log::trace!("Failed to get reply: {}", e);
                    continue;
                }
            };

            match self.proto_interface.send(reply, sender_addr) {
                Ok(_) => (),
                Err(e) => log::debug!("Failed to send reply: {}", e),
            }
        };

        if self.should_keep_running {
            log::error!("Node run loop exited unexpectedly");
            Err(Error::new(ErrorKind::Other, "Node run loop exited unexpectedly"))
        } else {
            log::info!("Server N{} shutting down...", self.id);
            Ok(())
        }
    }

    fn get_reply(&mut self, msg: UDPMessage) -> Result<Reply> {
        log::trace!("Entering get_reply: handling with Id of size {}", msg.id.len());

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

                if reply.status != (Status::OutOfMemory as u32) {
                    self.cache_reply(&msg.id, &reply)?;
                }

                reply
            }
        };

        log::trace!("Exiting get_reply: generated reply with status {}", reply.status);
        Ok(reply)
    }

    fn get_reply_from_cache(&mut self, msg: &UDPMessage) -> Result<Reply> {
        log::trace!("Entering get_reply_from_cache: handling with Id of size {}", msg.id.len());

        let reply: Result<Reply> = match self.request_cache.get(&msg.id) {
            Some(reply_bytes) => {
                log::debug!("Cache hit for message Id of size {}", msg.id.len());
                let reply: Reply = Reply::parse_from_bytes(reply_bytes.as_slice())?;
                Ok(reply)
            },
            None => {
                log::debug!("Cache miss for message Id of size {}", msg.id.len());
                Err(Error::new(ErrorKind::NotFound, "Cache miss"))
            },
        };

        log::trace!("Exiting get_reply_from_cache");
        reply
    }

    fn handle_message(&mut self, msg: &UDPMessage) -> Result<Reply> {
        log::trace!("Entering handle_message");

        let request: Request = extract_request(msg)?;
        let reply: Reply = match request.operation.try_into() {
            Ok(Operation::Put) => self.handle_put(request),
            Ok(Operation::Get) => self.handle_get(request),
            Ok(Operation::Delete) => self.handle_delete(request),
            Ok(Operation::Wipe) => self.handle_wipe(),
            Ok(Operation::Ping) => self.handle_ping(),
            Ok(Operation::Shutdown) => self.handle_shutdown(),
            Ok(Operation::GetPid) => self.handle_getpid(),
            _ => self.handle_undefined_operation(request.operation),
        };

        log::trace!("Exiting handle_message");
        Ok(reply)
    }

    fn handle_put(&mut self, request: Request) -> Reply {
        log::debug!("Entering handle_put");

        let mut reply: Reply = Reply::new();

        let key: Vec<u8> = match request.key {
            Some(key) => key,
            None => {
                log::debug!("PUT request MissingKey");
                reply.status = Status::MissingKey as u32;
                log::trace!("Exiting handle_put");
                return reply;
            }
        };

        let value: Vec<u8> = match request.value {
            Some(value) => value,
            None => {
                log::debug!("PUT request MissingValue");
                reply.status = Status::MissingValue as u32;
                log::trace!("Exiting handle_put");
                return reply;
            }
        };

        if value.len() > MAX_VALUE_PAYLOAD_SIZE_BYTES {
            log::debug!("PUT request InvalidValueSize. Value with size {} B exceeds the maximum of {} B",
                value.len(),
                MAX_VALUE_PAYLOAD_SIZE_BYTES);
            reply.status = Status::InvalidValueSize as u32;
            log::trace!("Exiting handle_put");
            return reply;
        }

        let key_value_mem_usage: u64 = (key.len() as u64) + (value.len() as u64);
        if self.data_store_mem_usage + key_value_mem_usage <= self.max_mem {
            log::debug!("PUT request Success (key size: {}, value size: {})", key.len(), value.len());
            self.data_store.insert(key, value);
            self.data_store_mem_usage += key_value_mem_usage;
            reply.status = Status::Success as u32;
        } else {
            log::info!("PUT request unsuccessful, hit memory limit");
            reply.status = Status::OutOfMemory as u32;
        }

        log::trace!("Exiting handle_put");
        reply
    }

    fn handle_get(&self, request: Request) -> Reply {
        log::trace!("Entering handle_get");

        let mut reply: Reply = Reply::new();

        let key: Vec<u8> = match request.key {
            Some(key) => key,
            None => {
                log::debug!("GET request MissingKey");
                reply.status = Status::MissingKey as u32;
                log::trace!("Exiting handle_get");
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
                log::trace!("Exiting handle_get");
                return reply;
            },
        };

        log::debug!("GET request Success (key size: {}, value size: {})", key.len(), value.len());
        reply.value = Some(value);


        log::trace!("Exiting handle_get");
        reply
    }

    fn handle_delete(&mut self, request: Request) -> Reply {
        log::trace!("Entering handle_delete");

        let mut reply: Reply = Reply::new();

        let key: Vec<u8> = match request.key {
            Some(key) => key,
            None => {
                log::debug!("DELETE request MissingKey");
                reply.status = Status::MissingKey as u32;
                log::trace!("Exiting handle_delete");
                return reply;
            }
        };

        let value: Vec<u8> = match self.data_store.remove(&key) {
            Some(value) => value,
            None => {
                log::debug!("DELETE request KeyNotFound");
                reply.status = Status::KeyNotFound as u32;
                log::trace!("Exiting handle_delete");
                return reply;
            },
        };
        self.data_store_mem_usage -= (key.len() as u64) + (value.len() as u64);

        log::debug!("DELETE request Success (key size: {})", key.len());
        reply.status = Status::Success as u32;
        reply.value = Some(value);

        log::trace!("Exiting handle_delete");
        reply
    }

    fn handle_wipe(&mut self) -> Reply {
        log::trace!("Entering handle_wipe");
        let mut reply: Reply = Reply::new();
        self.data_store = HashMap::new();
        self.data_store_mem_usage = 0;
        reply.status = Status::Success as u32;
        log::debug!("WIPE request Success");
        log::trace!("Exiting handle_wipe");
        reply
    }

    fn handle_ping(&self) -> Reply {
        log::trace!("Entering handle_ping");
        let mut reply: Reply = Reply::new();
        reply.status = Status::Success as u32;
        log::debug!("PING request Success");
        log::trace!("Exiting handle_ping");
        reply
    }

    fn handle_shutdown(&mut self) -> Reply {
        log::trace!("Entering handle_shutdown");
        let mut reply: Reply = Reply::new();
        self.should_keep_running = false;
        reply.status = Status::Success as u32;
        log::debug!("SHUTDOWN request Success");
        log::trace!("Exiting handle_shutdown");
        reply
    }

    fn handle_getpid(&self) -> Reply {
        log::trace!("Entering handle_getpid");
        let mut reply: Reply = Reply::new();
        reply.pid = Some(self.process_id);
        reply.status = Status::Success as u32;
        log::debug!("GETPID request Success");
        log::trace!("Exiting handle_getpid");
        reply
    }

    fn handle_undefined_operation(&self, bad_error_code: u32) -> Reply {
        log::trace!("Entering handle_undefined_operation");
        let mut reply: Reply = Reply::new();
        reply.status = Status::UndefinedOperation as u32;
        log::debug!("Undefined operation with code {}", bad_error_code);
        log::trace!("Exiting handle_undefined_operation");
        reply
    }

    fn handle_internal_error(&self) -> Reply {
        log::trace!("Entering handle_internal_error");
        let mut reply: Reply = Reply::new();
        reply.status = Status::InternalError as u32;
        log::debug!("Internal error");
        log::trace!("Exiting handle_internal_error");
        reply
    }

    fn cache_reply(&mut self, msg_id: &Vec<u8>, reply: &Reply) -> Result<()> {
        log::trace!("Entering cache_reply");
        match reply.write_to_bytes() {
            Ok(reply_bytes) => {
                let reply_size: u64 = reply_bytes.len() as u64;
                if self.get_current_memory_usage() + reply_size <= self.max_mem {
                    self.request_cache.insert(msg_id.to_vec(), reply_bytes);
                }
            },
            Err(e) => {
                log::error!("Failed to serialize reply: {}", e);
                log::trace!("Exiting cache_reply");
                return Err(Error::new(ErrorKind::InvalidData, "Failed to serialize reply"));
            }
        }

        log::trace!("Exiting cache_reply");
        Ok(())
    }

    fn get_current_memory_usage(&self) -> u64 {
        self.data_store_mem_usage + self.request_cache.weighted_size()
    }
}
