#[derive(Deserialize, Debug)]
pub struct PacketHeader {
    pub connection_id:  i64,
    pub action:         i32,
    pub transaction_id: i32,
}

#[derive(Debug, Serialize)]
pub struct ConnectionResponse {
    pub action:         i32,
    pub transaction_id: i32,
    pub connection_id:  i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerAnnounce {
    pub action:         i32,
    pub transaction_id: i32,
    pub interval:       i32,
    pub leechers:       i32,
    pub seeders:        i32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ClientAnnounce {
    pub info_hash:  [u8; 20], // 20
    pub peer_id:    [u8; 20], // 40
    pub downloaded: i64,      // 48
    pub remaining:  i64,      // 56
    pub uploaded:   i64,      // 64
    pub event:      i32,      // 68
    pub ip:         u32,      // 72
    pub key:        u32,      // 76
    pub num_want:   i32,      // 80
    pub port:       u16,      // 82
}

#[derive(Debug, Serialize)]
pub struct ServerError {
    pub action:         i32,
    pub transaction_id: i32,
    pub error:          String,
}
