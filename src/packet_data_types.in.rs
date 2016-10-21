#[derive(Debug, Deserialize)]
pub struct PacketHeader {
    pub connection_id:  i64,
    pub action:         i32,
    pub transaction_id: i32,
}

#[derive(Debug, Serialize)]
struct ConnectionResponse {
    action:         i32,
    transaction_id: i32,
    connection_id:  i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct ServerAnnounce {
    action:         i32,
    transaction_id: i32,
    interval:       i32,
    leechers:       i32,
    seeders:        i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientAnnounce {
    pub info_hash:  [u8; 20],
    pub peer_id:    [u8, 20],
    pub downloaded: i64,
    pub remaining:  i64,
    pub uploaded:   i64,
    pub event:      i32,
    pub ip:         u32,
    pub key:        u32,
    pub num_want:   i32,
    pub port:       u16,
}
