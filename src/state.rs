use std::sync::Arc;
use sqlx::PgPool;
use tokio::{sync::Mutex, time::Instant};

#[derive(Clone)]
pub struct IdStore {
    pub store: Arc<Mutex<Vec<PacketId>>>
}

pub struct PacketId {
    pub packet_id: String,
    pub timestamp: Instant
}

impl PacketId {
    pub fn new(packet_id: String) -> Self {
        PacketId {
            packet_id: packet_id,
            timestamp: Instant::now()
        }
    }
}

#[derive(Clone)]
pub struct PgState {
    pub pool: PgPool
}