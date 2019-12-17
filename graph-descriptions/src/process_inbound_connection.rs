use error::Error;
use uuid::Uuid;
use graph_description::ProcessInboundConnection;
use serde_json::Value;
use std::convert::TryFrom;
use node::NodeT;


pub enum ProcessInboundConnectionState {
    Bound,
    Existing,
    Closed,
}

impl From<ProcessInboundConnectionState> for u32 {
    fn from(p: ProcessInboundConnectionState) -> u32 {
        match p {
            ProcessInboundConnectionState::Bound => 1,
            ProcessInboundConnectionState::Closed => 2,
            ProcessInboundConnectionState::Existing => 3,
        }
    }
}

impl TryFrom<u32> for ProcessInboundConnectionState {
    type Error = Error;

    fn try_from(p: u32) -> Result<ProcessInboundConnectionState, Error> {
        match p {
            1 => Ok(ProcessInboundConnectionState::Bound),
            2 => Ok(ProcessInboundConnectionState::Closed),
            3 => Ok(ProcessInboundConnectionState::Existing),
            _ => Err(Error::InvalidProcessInboundConnectionState(p))
        }
    }
}

impl ProcessInboundConnection {
    pub fn new(
        asset_id: impl Into<Option<String>>,
        hostname: impl Into<Option<String>>,
        state: ProcessInboundConnectionState,
        port: u16,
        ip_address: impl Into<String>,
        created_timestamp: u64,
        terminated_timestamp: u64,
        last_seen_timestamp: u64,
    ) -> Self {
        let asset_id = asset_id.into();
        let hostname = hostname.into();

        if hostname.is_none() && asset_id.is_none() {
            panic!("ProcessInboundConnection must have at least asset_id or hostname");
        }

        let ip_address = ip_address.into();

        Self {
            node_key: Uuid::new_v4().to_string(),
            ip_address,
            asset_id,
            hostname,
            created_timestamp,
            terminated_timestamp,
            last_seen_timestamp,
            port: port as u32,
            state: state.into(),
        }
    }

    pub fn into_json(self) -> Value {
        json!({
            "node_key": self.node_key,
            "dgraph.type": "ProcessInboundConnection",
        })
    }
}

impl NodeT for ProcessInboundConnection {
    fn get_asset_id(&self) -> Option<&str> {
        None
    }

    fn set_asset_id(&mut self, _asset_id: impl Into<String>) {
        panic!("Can not set asset_id on ProcessInboundConnection");
    }

    fn get_node_key(&self) -> &str {
        &self.node_key
    }

    fn set_node_key(&mut self, node_key: impl Into<String>) {
        self.node_key = node_key.into();
    }

    fn merge(&mut self, other: &Self) -> bool {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two IpAddress Nodes with differing node_keys");
            return false
        }

        if self.ip_address != other.ip_address {
            warn!("Attempted to merge two IpAddress Nodes with differing IPs");
            return false;
        }

        let mut merged = false;

        if self.asset_id.is_none() && other.asset_id.is_some() {
            self.asset_id = other.asset_id.clone();
        }

        if self.hostname.is_none() && other.hostname.is_some() {
            self.hostname = other.hostname.clone();
        }

        if self.created_timestamp != 0 && self.created_timestamp > other.created_timestamp {
            self.created_timestamp = other.created_timestamp;
            merged = true;
        }

        if self.terminated_timestamp != 0 && self.terminated_timestamp < other.terminated_timestamp {
            self.terminated_timestamp = other.terminated_timestamp;
            merged = true;
        }

        if self.last_seen_timestamp != 0 && self.last_seen_timestamp < other.last_seen_timestamp {
            self.last_seen_timestamp = other.last_seen_timestamp;
            merged = true;
        }

        merged
    }

    fn merge_into(&mut self, other: Self) -> bool {
        self.merge(&other)
    }
}