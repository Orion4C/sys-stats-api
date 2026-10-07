use uuid::Uuid;

pub fn new_session_id() -> Uuid {
    Uuid::now_v7()
}

pub fn session_key(session: Uuid, seq: u64) -> [u8; 24] {
    let mut key = [0u8; 24];
    key[..16].copy_from_slice(session.as_bytes());
    key[16..].copy_from_slice(&seq.to_be_bytes());
    key
}
