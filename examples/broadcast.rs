//! This example demonstrates how to use the library to encode and decode data in a broadcast
//! scenario, with some intermediate nodes.

use rlnc::{decode::Decoder, encode::Encoder, primitives::RLNCPacket};

struct Node {
    id: u64,
    peers: Vec<u64>,
    upload_bandwidth: u64,
    download_bandwidth: u64,
    packets: Vec<RLNCPacket>,
    mesh_degree: u32,
    encoder: Encoder,
    decoder: Decoder,
}

impl Node {
    fn new(id: u64, upload_bandwidth: u64, download_bandwidth: u64, mesh_degree: u32) -> Self {
        Self {
            id,
            peers: vec![],
            upload_bandwidth,
            download_bandwidth,
            packets: vec![],
            mesh_degree,
        }
    }

    fn add_peer(&mut self, peer: u64) {
        self.peers.push(peer);
    }
}

struct Network {
    nodes: Vec<Node>,
    bandwidth: u64,
}

impl Network {
    fn build_tree(&self, size: usize) {}

    fn build_random(&self, size: usize) {}
}

fn main() {}
