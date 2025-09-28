use std::{collections::HashMap, io::Result};
use std::net::SocketAddr;

use crate::util;

pub struct FingerTable {
    finger_start_chord_positions: Vec<u32>,
    membership_list: HashMap<SocketAddr, u32>,
    size_factor: usize
}

impl FingerTable {
    pub fn new(socket_addr: SocketAddr, peer_socket_addrs: Vec<SocketAddr>, size_factor: usize) -> Result<Self> {

        let mut finger_start_chord_positions: Vec<u32> = Vec::new();

        let node_position: u32 = FingerTable::calculate_member_position_from_address(&socket_addr, size_factor);
        finger_start_chord_positions.push(node_position);

        // Start at index 1 because position 0 was already calculated
        for finger_index in 1..size_factor {
            let next_position: u32 = FingerTable::calculate_start_position(node_position, finger_index, size_factor);
            finger_start_chord_positions.push(next_position);
        }

        // TODO: Instead of storing the membership list, use the initial list to get the successors of each finger

        // FIXME Should probably use a different server naming convention other than IP address
        // in the scenario the IP address changes.
        let mut membership_list: HashMap<SocketAddr, u32> = HashMap::new();
        for peer_addr in peer_socket_addrs.iter() {
            let peer_position: u32 = FingerTable::calculate_member_position_from_address(peer_addr, size_factor);
            membership_list.insert(socket_addr, peer_position);
        }

        Ok(FingerTable { finger_start_chord_positions: finger_start_chord_positions, membership_list: membership_list, size_factor: size_factor })
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
        self.get_start_position(0)
    }

    fn get_start_position(&self, finger_index: usize) -> u32 {
        self.finger_start_chord_positions[finger_index]
    }

    fn get_finger_table_size(&self) -> usize {
        self.finger_start_chord_positions.len()
    }

    // TODO Implement function to find the first node greater than or equal to the given finger

    // TODO Implement function to find the successor

    // TODO Implement function to find the predecessor

    // Static functions
    fn calculate_member_position_from_address(socket_addr: &SocketAddr, size_factor: usize) -> u32 {
        // FIXME: May need to distinguish between local IP and public IP for the socket address
        // of the current node to ensure that the hashing of each node remains consistent
        // across all nodes. For now, assuming they will always be on local host.

        // Should be fine to unwrap since the finger table is initialized once at startup
        let socket_addr_hash: [u8; 16] = util::hash_md5(socket_addr.to_string()).unwrap();
        let socket_addr_hash_as_uint: u128 = u128::from_be_bytes(socket_addr_hash);

        const BASE: i32 = 2;
        let max_key_plus_one: u128 = BASE.pow(size_factor as u32) as u128;

        // FIXME: This restricts the size factor to have a max of 32, any larger will truncate
        // calculated node positions. Might need to fix this later.
        (socket_addr_hash_as_uint % max_key_plus_one) as u32
    }

    fn calculate_start_position(first_finger_position: u32, finger_index: usize, size_factor: usize) -> u32 {
        const BASE: i32 = 2;
        let offset_from_first_finger: u32 = BASE.pow(finger_index as u32) as u32;
        let max_key_plus_one: u32 = BASE.pow(size_factor as u32) as u32;
        (first_finger_position + offset_from_first_finger) % max_key_plus_one
    }
}
