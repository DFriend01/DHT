use std::io::Result;
use std::net::SocketAddr;

use crate::util;

pub struct FingerTable {
    finger_start_positions: Vec<u32>,
    finger_node_positions: Vec<u32>,
    finger_node_socket_addrs: Vec<SocketAddr>
}

impl FingerTable {
    pub fn new(this_node_socket_addr: SocketAddr, peer_socket_addrs: Vec<SocketAddr>, size_factor: usize) -> Result<Self> {
        let finger_start_positions: Vec<u32> = FingerTable::get_finger_start_positions(&this_node_socket_addr, size_factor);
        let (finger_node_positions, finger_node_socket_addrs) = FingerTable::get_finger_node_positions_and_addrs(
            this_node_socket_addr,
            peer_socket_addrs,
            finger_start_positions.clone(),
            size_factor
        );

        Ok(FingerTable {
            finger_start_positions: finger_start_positions,
            finger_node_positions: finger_node_positions,
            finger_node_socket_addrs: finger_node_socket_addrs
        })
    }

    // Private functions
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

    fn get_start_position(&self, finger_index: usize) -> u32 {
        self.finger_start_positions[finger_index]
    }

    fn get_node_position(&self, finger_index: usize) -> u32 {
        self.finger_node_positions[finger_index]
    }

    fn get_node_address(&self, finger_index: usize) -> SocketAddr {
        self.finger_node_socket_addrs[finger_index]
    }

    fn get_successor_position(&self) -> u32 {
        const SECOND_FINGER: usize = 1;
        self.get_node_position(SECOND_FINGER)
    }

    fn get_finger_table_size(&self) -> usize {
        self.finger_start_positions.len()
    }

    // TODO Implement function to find the predecessor

    // Static functions
    fn get_finger_start_positions(node_socket_addr: &SocketAddr, size_factor: usize) -> Vec<u32> {
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

    fn get_finger_node_positions_and_addrs(node_socket_addr: SocketAddr,
        peer_socket_addrs: Vec<SocketAddr>,
        finger_start_positions: Vec<u32>,
        size_factor: usize) -> (Vec<u32>, Vec<SocketAddr>) {

        let (sorted_peer_positions, sorted_peer_socket_addrs) = FingerTable::get_sorted_peer_positions_and_addrs(peer_socket_addrs, size_factor);

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

    fn get_sorted_peer_positions_and_addrs(peer_socket_addrs: Vec<SocketAddr>, size_factor: usize) -> (Vec<u32>, Vec<SocketAddr>) {
        // FIXME Should probably use a different server naming convention other than IP address
        // in the scenario the IP address changes.
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
        // calculated node positions. Might need to fix this later.
        (socket_addr_hash_as_uint % max_position_plus_one) as u32
    }

    fn calculate_start_position(first_finger_position: u32, finger_index: usize, size_factor: usize) -> u32 {
        const BASE: i32 = 2;
        let offset_from_first_finger: u32 = BASE.pow(finger_index as u32) as u32;
        let max_position_plus_one: u32 = BASE.pow(size_factor as u32) as u32;
        (first_finger_position + offset_from_first_finger) % max_position_plus_one
    }
}
