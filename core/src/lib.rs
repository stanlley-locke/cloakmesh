pub mod config;
pub mod crypto;
pub mod errors;
pub mod network;
pub mod routing;
pub mod cloak_protocol;
pub mod ledger;
pub mod storage;
pub mod telemetry;
pub mod types;

pub mod node;
pub mod proto {
    pub mod v1 {
        tonic::include_proto!("cloakmesh.v1");
    }
}

#[cfg(test)]
mod proto_smoke_test;
