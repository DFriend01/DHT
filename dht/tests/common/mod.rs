use std::io::{Error, ErrorKind, Result};
use std::net::{SocketAddr, UdpSocket};

use dht::comm::ProtoInterface;
use dht::comm::proto::{extract_reply, Operation, Status};
use dht::comm::protogen::api::{Request, Reply};

pub fn ping_servers(server_addrs: Vec<SocketAddr>, should_panic_if_fail: bool) -> Result<()> {
    let client_addr: SocketAddr = UdpSocket::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();

    let proto_interface = match ProtoInterface::new(client_addr) {
        Ok(proto_interface) => proto_interface,
        Err(e) => {
            if should_panic_if_fail {
                panic!("Failed to create ProtoInterface: {:?}", e);
            } else {
                return Err(e);
            }
        }
    };

    for server_addr in server_addrs {
        let mut request: Request = Request::new();
        request.operation = Operation::Ping as u32;
        let (reply_msg, _) = match proto_interface.send_and_recv(request, server_addr) {
            Ok((reply, _server_socket)) => (reply, _server_socket),
            Err(e) => {
                if should_panic_if_fail {
                    panic!("Ping failed: {:?}", e);
                } else {
                    return Err(e);
                }
            }
        };

        let reply: Reply = match extract_reply(&reply_msg) {
            Ok(reply) => reply,
            Err(e) => {
                if should_panic_if_fail {
                    panic!("Ping failed: {:?}", e);
                } else {
                    return Err(e);
                }
            }
        };

        if reply.status != Status::Success as u32 {
            if should_panic_if_fail {
                panic!("Ping failed: {:?}", reply.status);
            } else {
                return Err(Error::new(ErrorKind::Other, "Ping did not return SUCCESS"));
            }
        }
    }
    Ok(())
}
