//   Copyright 2017 Justin Noah <justinnoah@gmail.com>
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.


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
