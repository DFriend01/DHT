#![allow(unreachable_code)]

use std::io::{Result, Error, ErrorKind};
use std::net::SocketAddr;
use std::collections::HashMap;
use log;

use crate::comm::ProtoInterface;
use crate::comm::proto::{Operation, Status, extract_request};
use crate::comm::protogen::api::{UDPMessage, Request, Reply};

pub struct Node {
    proto_interface: ProtoInterface,
    data_store: HashMap<Vec<u8>, Vec<u8>>,
    id: u32,
}

impl Node {
    pub fn new(socket_addr: SocketAddr, id: u32) -> Result<Self> {
        let proto_interface: ProtoInterface = ProtoInterface::new(socket_addr)?;
        let data_store: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        Ok(Node {proto_interface, data_store, id})
    }

    pub fn run(&mut self) -> Result<()> {
        log::info!("Server N{} starting up...", self.id);

        loop {
            let (msg, sender_addr) = match self.proto_interface.listen() {
                Ok((msg, addr)) => (msg, addr),
                Err(e) => {
                    eprintln!("Failed to receive message: {}", e);
                    continue;
                }
            };

            let reply = self.handle_message(msg)?;

            match self.proto_interface.send(reply, sender_addr) {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to send reply: {}", e),
            }
        };

        log::error!("Node run loop exited unexpectedly");
        Err(Error::new(ErrorKind::Other, "Node run loop exited unexpectedly"))
    }

    fn handle_message(&mut self, msg: UDPMessage) -> Result<Reply> {
        let request: Request = extract_request(msg)?;

        if let Ok(Operation::Shutdown) = request.operation.try_into() {
            std::process::exit(0);
        }

        let reply: Reply = match request.operation.try_into() {
            Ok(Operation::Put) => self.handle_put(request),
            Ok(Operation::Get) => self.handle_get(request),
            Ok(Operation::Delete) => self.handle_delete(request),
            Ok(Operation::Wipe) => self.handle_wipe(),
            Ok(Operation::Ping) => self.handle_ping(),
            _ => self.handle_undefined_operation()
        };

        Ok(reply)
    }

    fn handle_put(&mut self, request: Request) -> Reply {
        let mut reply: Reply = Reply::new();

        let key: Vec<u8> = match request.key {
            Some(key) => key,
            None => {
                reply.status = Status::MissingKey as u32;
                return reply;
            }
        };

        let value: Vec<u8> = match request.value {
            Some(value) => value,
            None => {
                reply.status = Status::MissingValue as u32;
                return reply;
            }
        };

        self.data_store.insert(key, value);
        reply.status = Status::Success as u32;
        reply
    }

    fn handle_get(&self, request: Request) -> Reply {
        let mut reply: Reply = Reply::new();

        let key: Vec<u8> = match request.key {
            Some(key) => key,
            None => {
                reply.status = Status::MissingKey as u32;
                return reply;
            }
        };

        match self.data_store.get(&key) {
            Some(value) => {
                reply.status = Status::Success as u32;
                reply.value = Some(value.to_vec());
            },
            None => reply.status = Status::KeyNotFound as u32,
        };

        reply
    }

    fn handle_delete(&mut self, request: Request) -> Reply {
        let mut reply: Reply = Reply::new();

        let key: Vec<u8> = match request.key {
            Some(key) => key,
            None => {
                reply.status = Status::MissingKey as u32;
                return reply;
            }
        };

        match self.data_store.remove(&key) {
            Some(value) => {
                reply.status = Status::Success as u32;
                reply.value = Some(value);
            },
            None => reply.status = Status::KeyNotFound as u32,
        };

        reply
    }

    fn handle_wipe(&mut self) -> Reply {
        let mut reply: Reply = Reply::new();
        self.data_store = HashMap::new();
        reply.status = Status::Success as u32;
        reply
    }

    fn handle_ping(&self) -> Reply {
        let mut reply: Reply = Reply::new();
        reply.status = Status::Success as u32;
        reply
    }

    fn handle_undefined_operation(&self) -> Reply {
        let mut reply: Reply = Reply::new();
        reply.status = Status::UndefinedOperation as u32;
        reply
    }
}
