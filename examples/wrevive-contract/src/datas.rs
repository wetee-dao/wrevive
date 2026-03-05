use parity_scale_codec::{Decode, Encode};
use wrevive_api::{AccountId, Address, BlockNumber, Bytes, Vec};

/// K8s worker 节点信息（Subnet::worker 返回值等）
#[derive(Default, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct Cluster {
    pub name: Bytes,
    pub owner: Address,
    pub level: u8,
    pub region_id: u32,
    pub start_block: BlockNumber,
    pub stop_block: Option<BlockNumber>,
    pub terminal_block: Option<BlockNumber>,
    pub p2p_id: AccountId,
    pub ip: Ip,
    pub port: u32,
    pub status: u8,
}

/// Ip
#[derive(Default, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct Ip {
    pub ipv4: Option<u32>,
    pub ipv6: Option<u128>,
    pub domain: Option<Vec<u8>>,
}
