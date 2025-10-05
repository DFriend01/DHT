use std::io::{Result, Error, ErrorKind};
use std::net::SocketAddr;
use log;

use crate::comm::proto::{extract_reply, Operation, Status};
use crate::comm::protogen::api::{NodeInfo, Request, Reply};
use crate::comm::ProtoInterface;
use crate::util;

pub struct FingerTable {
    finger_start_positions: Vec<u32>,
    finger_node_positions: Vec<u32>,
    finger_node_socket_addrs: Vec<SocketAddr>
}

/* TODO Functions to implement for later iterations:
    - Function(s) to update the finger table based on node joins and leaves
*/

impl FingerTable {
    pub fn new(this_node_socket_addr: SocketAddr, peer_socket_addrs: Vec<SocketAddr>, size_factor: usize) -> Result<Self> {

        const BASE: i32 = 2;
        assert!(
            peer_socket_addrs.len() + 1 <= BASE.pow(size_factor as u32) as usize,
            "The underlying chord structure does not have enough space for all nodes"
        );

        let finger_start_positions: Vec<u32> = FingerTable::calculate_start_positions(&this_node_socket_addr, size_factor);
        let (finger_node_positions, finger_node_socket_addrs) = FingerTable::calculate_node_positions_and_addrs(
            this_node_socket_addr,
            peer_socket_addrs,
            finger_start_positions.clone(),
            size_factor
        );

        let mut log_message: String = String::new();
        log_message.push_str("FingerTable initial state (Start Position,Node Position,Node Address): ");
        for i in 0..finger_node_positions.len() {
            let start_position: &u32 = finger_start_positions.get(i).expect("finger_start_positions was expected to have all values to be non-empty");
            let node_position: &u32 = finger_node_positions.get(i).expect("finger_node_positions was expected to have all values to be non-empty");
            let node_address: &SocketAddr = finger_node_socket_addrs.get(i).expect("finger_node_socket_addrs was expected to have all values to be non-empty");
            log_message.push_str(format!("({},{},{})", start_position, node_position, node_address.to_string()).as_str());

            if i < finger_node_positions.len() - 1 {
                log_message.push_str(", ");
            }
        }
        log::info!("{}", log_message);

        Ok(FingerTable {
            finger_start_positions: finger_start_positions,
            finger_node_positions: finger_node_positions,
            finger_node_socket_addrs: finger_node_socket_addrs
        })
    }

    // Public functions
    pub fn find_successor_of_key(&self, key: Vec<u8>) -> Result<SocketAddr> {
        let predecessor_address: SocketAddr = self.find_predecessor_of_key(key)?;
        let (_, successor_address) = self.get_node_successor(predecessor_address)?;
        Ok(successor_address)
    }

    pub fn find_nearest_preceding_finger(&self, key: Vec<u8>) -> Result<usize> {
        let key_position: u32 = self.calculate_key_position(key)?;
        let node_position: u32 = self.get_position_of_this_node();
        const NODE_INCLUSIVE: bool = false;
        const KEY_INCLUSIVE: bool = false;
        let max_position_plus_one: u32 = self.get_max_position() + 1;

        for finger_index in (0..self.get_finger_table_size()).rev() {
            let finger_node_position: u32 = self.get_node_position(finger_index);
            let is_finger_between_node_and_key: bool = util::is_in_wraparound_range(
                node_position,
                key_position,
                finger_node_position,
                max_position_plus_one,
                NODE_INCLUSIVE,
                KEY_INCLUSIVE
            );
            if is_finger_between_node_and_key {
                return Ok(finger_index);
            }
        }

        Err(Error::new(ErrorKind::NotFound, "Nearest finger not found, but we should not be seeing this message..."))
    }

    pub fn get_finger_start_positions(&self) -> Vec<u32> {
        self.finger_node_positions.clone()
    }

    pub fn get_finger_node_positions(&self) -> Vec<u32> {
        self.finger_node_positions.clone()
    }

    pub fn get_finger_node_socket_addrs(&self) -> Vec<SocketAddr> {
        self.finger_node_socket_addrs.clone()
    }

    pub fn get_node_position(&self, finger_index: usize) -> u32 {
        self.finger_node_positions[finger_index]
    }

    pub fn get_node_address(&self, finger_index: usize) -> SocketAddr {
        self.finger_node_socket_addrs[finger_index]
    }

    pub fn get_successor_position_of_this_node(&self) -> u32 {
        const SECOND_FINGER: usize = 1;
        self.get_node_position(SECOND_FINGER)
    }

    pub fn get_successor_addr_of_this_node(&self) -> SocketAddr {
        const SECOND_FINGER: usize = 1;
        self.get_node_address(SECOND_FINGER)
    }

    // Private functions
    fn find_predecessor_of_key(&self, key: Vec<u8>) -> Result<SocketAddr> {
        let key_position: u32 = self.calculate_key_position(key.clone())?;
        let node_position: u32 = self.get_position_of_this_node();
        let node_successor_position: u32 = self.get_successor_position_of_this_node();

        let max_position_plus_one: u32 = self.get_max_position() + 1;
        const NODE_INCLUSIVE: bool = false;
        const NODE_SUCCESSOR_INCLUSIVE: bool = true;

        // Does the key already belong to this node?
        let key_belongs_to_this_node: bool  = util::is_in_wraparound_range(
            node_position,
            node_successor_position,
            key_position,
            max_position_plus_one,
            NODE_INCLUSIVE,
            NODE_SUCCESSOR_INCLUSIVE
        );

        if key_belongs_to_this_node {
            return Ok(self.get_addr_of_this_node());
        }

        // Does the key belong to one of the fingers on this node?
        let index_of_nearest_preceding_finger: usize = self.find_nearest_preceding_finger(key.clone())?;
        let finger_node_position: u32 = self.get_node_position(index_of_nearest_preceding_finger);
        let finger_node_address: SocketAddr = self.get_node_address(index_of_nearest_preceding_finger);
        let (finger_node_successor_position, _) = self.get_node_successor(finger_node_address)?;

        let key_belongs_to_finger_node: bool = util::is_in_wraparound_range(
            finger_node_position,
            finger_node_successor_position,
            key_position,
            max_position_plus_one,
            NODE_INCLUSIVE,
        NODE_SUCCESSOR_INCLUSIVE
        );

        if key_belongs_to_finger_node {
            return Ok(self.get_node_address(index_of_nearest_preceding_finger));
        }

        // Search the finger tables from other nodes to map the key.
        // The most hops we would ever need to make in an N-node network is
        // O(log N). Since we do not always know the number of nodes,
        // we use the size of the chord: log(2 ^ finger_table_size) = finger_table_size
        // as an upper bound.
        let max_node_hops: usize = self.get_finger_table_size();

        let socket: SocketAddr = util::get_randomly_available_socket();
        let proto_interface: ProtoInterface = ProtoInterface::new(socket)?;

        let mut next_peer_addr: SocketAddr = self.get_node_address(index_of_nearest_preceding_finger);
        for hop in 0..max_node_hops {
            let mut search_request: Request = Request::new();
            search_request.operation = Operation::GetNearestPrecedingNodeToKey as u32;
            search_request.key = Some(key.clone());

            log::debug!("Hop {} in chord to map key to node, contacting node with address {}", hop + 1, next_peer_addr);
            let (reply_msg, _server_socket) = proto_interface.send_and_recv(search_request, next_peer_addr)?;
            let reply: Reply = extract_reply(&reply_msg)?;

            if reply.status != Status::Success as u32 {
                break;
            }

            let results: NodeInfo = match reply.node_info.into_option() {
                Some(search_results) => search_results,
                None => break
            };

            let nearest_preceding_node_position: u32 = results.node_position;
            let nearest_preceding_node_addr: SocketAddr = match results.node_address.parse() {
                Ok(address) => address,
                Err(_) => break
            };

            let (nearest_preceding_node_successor_position, _) = match self.get_node_successor(nearest_preceding_node_addr) {
                Ok((position, addr)) => (position, addr),
                Err(_) => break
            };

            let is_key_predecessor_found: bool = util::is_in_wraparound_range(
                nearest_preceding_node_position,
                nearest_preceding_node_successor_position,
                key_position,
                max_position_plus_one,
                NODE_INCLUSIVE,
                NODE_SUCCESSOR_INCLUSIVE
            );

            if is_key_predecessor_found {
                return Ok(nearest_preceding_node_addr)
            } else {
                next_peer_addr = nearest_preceding_node_addr;
            }
        }

        Err(Error::new(ErrorKind::NotFound, "Node not found for key"))
    }

    fn get_node_successor(&self, node_addr: SocketAddr) -> Result<(u32, SocketAddr)> {
        let is_this_node: bool = node_addr == self.get_addr_of_this_node();
        if is_this_node {
            return Ok((self.get_successor_position_of_this_node(), self.get_successor_addr_of_this_node()));
        }

        let mut request: Request = Request::new();
        request.operation = Operation::GetSuccessor as u32;

        let socket: SocketAddr = util::get_randomly_available_socket();
        let proto_interface: ProtoInterface = ProtoInterface::new(socket)?;

        let (reply_msg, _server_socket) = proto_interface.send_and_recv(request, node_addr)?;
        let reply: Reply = extract_reply(&reply_msg)?;

        if reply.status != Status::Success as u32 {
            return Err(Error::new(ErrorKind::Other, "GetSuccessor failed"));
        }

        let successor_info: NodeInfo = match reply.node_info.into_option() {
            Some(info) => info,
            None => {
                return Err(Error::new(ErrorKind::Other, "GetSuccessor node information was empty!"));
            }
        };

        let successor_position: u32 = successor_info.node_position;
        let successor_address: SocketAddr = match successor_info.node_address.parse() {
            Ok(address) => address,
            Err(_) => {
                return Err(Error::new(ErrorKind::InvalidData, "Unable to parse socket address"));
            }
        };

        Ok((successor_position, successor_address))

    }

    fn is_key_in_finger_interval(&self, key_position: u32, finger_index: usize) -> bool {
        let finger_interval: [u32; 2] = self.get_position_interval(finger_index);
        let start: u32 = finger_interval[0];
        let end: u32 = finger_interval[1];
        if start == end {
            key_position == start
        } else if start < end {
            (key_position >= start) && (key_position < end)
        } else {
            ((key_position >= start) && (key_position <= self.get_max_position())) || (key_position < end)
        }
    }

    fn get_position_interval(&self, finger_index: usize) -> [u32; 2] {
        let next_finger_index: usize = (finger_index + 1) % self.get_finger_table_size();

        let interval_start: u32 = self.get_start_position(finger_index);
        let interval_end: u32 = self.get_start_position(next_finger_index);

        // [Inclusive start, exclusive end)
        let mut interval: [u32; 2] = [0; 2];
        interval[0] = interval_start;
        interval[1] = interval_end;

        interval
    }

    fn get_position_of_this_node(&self) -> u32 {
        const FIRST_FINGER: usize = 0;
        self.get_start_position(FIRST_FINGER)
    }

    fn get_addr_of_this_node(&self) -> SocketAddr {
        const FIRST_FINGER: usize = 0;
        self.get_node_address(FIRST_FINGER)
    }

    fn get_start_position(&self, finger_index: usize) -> u32 {
        self.finger_start_positions[finger_index]
    }

    fn get_finger_table_size(&self) -> usize {
        self.finger_start_positions.len()
    }

    fn calculate_key_position(&self, key: impl AsRef<[u8]>) -> Result<u32> {
        let hash: [u8; 16] = match util::hash_md5(key) {
            Ok(hash) => hash,
            Err(_e) => {
                return Err(Error::new(ErrorKind::Other, "Hash failed"))
            }
        };

        let hash_uint: u128 = u128::from_be_bytes(hash);
        let max_position_plus_one: u128 = (self.get_max_position() as u128) + 1;
        Ok((hash_uint % max_position_plus_one) as u32)
    }

    fn get_max_position(&self) -> u32 {
        const BASE: i32 = 2;
        let size_factor: usize = self.get_finger_table_size();

        // Need to be careful about truncation here by limiting the size factor...
        let max_position_plus_one: u128 = BASE.pow(size_factor as u32) as u128;
        (max_position_plus_one - 1) as u32
    }

    // Static functions
    fn calculate_start_positions(node_socket_addr: &SocketAddr, size_factor: usize) -> Vec<u32> {
        let mut finger_start_positions: Vec<u32> = Vec::new();

        // The first finger has its start position the same as this node's position
        let first_finger_position: u32 = FingerTable::calculate_node_position_from_address(node_socket_addr, size_factor);
        finger_start_positions.push(first_finger_position);

        // Now, get the start position for the rest of the fingers
        for finger_index in 1..size_factor {
            let finger_start_position: u32 = FingerTable::calculate_start_position(first_finger_position, finger_index, size_factor);
            finger_start_positions.push(finger_start_position);
        }

        finger_start_positions
    }

    fn calculate_node_positions_and_addrs(node_socket_addr: SocketAddr,
        peer_socket_addrs: Vec<SocketAddr>,
        finger_start_positions: Vec<u32>,
        size_factor: usize) -> (Vec<u32>, Vec<SocketAddr>) {

        let (sorted_peer_positions, sorted_peer_socket_addrs) = FingerTable::calculate_sorted_peer_positions_and_addrs(peer_socket_addrs, size_factor);

        let mut finger_node_positions: Vec<u32> = Vec::new();
        let mut finger_node_addrs: Vec<SocketAddr> = Vec::new();

        // The first finger is this node
        let first_finger_position: u32 = finger_start_positions[0];
        finger_node_positions.push(first_finger_position);
        finger_node_addrs.push(node_socket_addr);

        // Now, find the rest of the fingers
        let mut finger_index: usize = 1;
        let mut peer_index: usize = 0;

        while (finger_index < finger_start_positions.len()) && (peer_index < sorted_peer_positions.len()) {
            let finger_start_position: u32 = finger_start_positions[finger_index];
            let peer_position: u32 = sorted_peer_positions[peer_index];

            if peer_position >= finger_start_position {
                finger_node_positions.push(peer_position);
                finger_node_addrs.push(sorted_peer_socket_addrs[peer_index]);
                finger_index += 1;
            } else {
                peer_index += 1;
            }
        }

        (finger_node_positions, finger_node_addrs)
    }

    fn calculate_sorted_peer_positions_and_addrs(peer_socket_addrs: Vec<SocketAddr>, size_factor: usize) -> (Vec<u32>, Vec<SocketAddr>) {
        // FIXME Should probably use a different server naming convention other than IP address
        // in the scenario the IP address changes. Also, how to handle name conflicts in the event
        // that the name hash is the same?
        let mut peer_positions: Vec<u32> = Vec::new();
        for peer_addr in peer_socket_addrs.iter() {
            let peer_position: u32 = FingerTable::calculate_node_position_from_address(peer_addr, size_factor);
            peer_positions.push(peer_position);
        }

        let mut indices: Vec<usize> = (0..peer_positions.len()).collect();
        indices.sort_by_key(|&i| peer_positions[i]);

        let sorted_peer_positions: Vec<u32> = indices.iter().map(|&i| peer_positions[i]).collect();
        let sorted_peer_socket_addrs: Vec<SocketAddr> = indices.iter().map(|&i| peer_socket_addrs[i]).collect();

        (sorted_peer_positions, sorted_peer_socket_addrs)
    }

    fn calculate_node_position_from_address(socket_addr: &SocketAddr, size_factor: usize) -> u32 {
        // FIXME: May need to distinguish between local IP and public IP for the socket address
        // of the current node to ensure that the hashing of each node remains consistent
        // across all nodes. For now, assuming they will always be on local host.

        // Should be fine to unwrap since the finger table is initialized once at startup
        let socket_addr_hash: [u8; 16] = util::hash_md5(socket_addr.to_string()).unwrap();
        let socket_addr_hash_as_uint: u128 = u128::from_be_bytes(socket_addr_hash);

        const BASE: i32 = 2;
        let max_position_plus_one: u128 = BASE.pow(size_factor as u32) as u128;

        // FIXME: This restricts the size factor to have a max of 32, any larger will truncate
        // calculated key/node positions. Might need to fix this later.
        (socket_addr_hash_as_uint % max_position_plus_one) as u32
    }

    fn calculate_start_position(first_finger_position: u32, finger_index: usize, size_factor: usize) -> u32 {
        const BASE: i32 = 2;
        let offset_from_first_finger: u32 = BASE.pow(finger_index as u32) as u32;
        let max_position_plus_one: u32 = BASE.pow(size_factor as u32) as u32;
        (first_finger_position + offset_from_first_finger) % max_position_plus_one
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_single_node() {
        let node_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let peer_addrs: Vec<SocketAddr> = Vec::new();
        const SIZE_FACTOR: usize = 1;

        let finger_table: FingerTable = FingerTable::new(node_addr, peer_addrs, SIZE_FACTOR).unwrap();

        let finger_start_positions: Vec<u32> = finger_table.get_finger_start_positions();
        assert_eq!(finger_start_positions.len(), 1);

        let finger_node_positions: Vec<u32> = finger_table.get_finger_node_positions();
        assert_eq!(finger_node_positions.len(), 1);
        assert_eq!(finger_start_positions, finger_node_positions);

        let finger_node_addrs: Vec<SocketAddr> = finger_table.get_finger_node_socket_addrs();
        assert_eq!(finger_node_addrs.len(), 1);
        assert_eq!(finger_node_addrs[0], node_addr);
    }

    #[test]
    fn test_multiple_nodes() {
        let node_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut peer_addrs: Vec<SocketAddr> = Vec::new();

        const NUM_PEERS: usize = 10;
        for i in 0..NUM_PEERS {
            let peer_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
            peer_addrs.push(peer_addr);
        }

        const SIZE_FACTOR: usize = 8;
        let finger_table: FingerTable = FingerTable::new(node_addr, peer_addrs, SIZE_FACTOR).unwrap();
    }
}
