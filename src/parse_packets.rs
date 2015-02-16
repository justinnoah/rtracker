// Copyright 2015 Justin Noah, All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate bincode;
extern crate "rustc-serialize" as rustc_serialize;

use self::bincode::SizeLimit;

pub struct PacketHeader {
    pub connection_id:  u64,
    pub action:         u32,
    pub transaction_id: u32,
}

#[derive(RustcEncodable)]
struct ConnectionResponse {
    action:         u32,
    transaction_id: u32,
    connection_id:  u64,
}

pub fn parse_header(packet: &[u8]) -> PacketHeader {
    let p_con_id            = &packet[0..8];
    let p_action            = &packet[8..12];
    let p_tran_id           = &packet[12..16];

    let mut connection_id:  u64 = 0;
    let mut action:         u32 = 0;
    let mut transaction_id: u32 = 0;

    for i in p_con_id.iter() {
        connection_id <<= 8;
        connection_id |= (*i as u64);
    }

    for i in p_action.iter() {
        action <<= 8;
        action |= (*i as u32);
    }

    for i in p_tran_id.iter() {
        transaction_id <<= 8;
        transaction_id |= (*i as u32);
    }

    PacketHeader {connection_id: connection_id, action: action, transaction_id: transaction_id}
}

pub fn encode_connect_response(uuid: u64, tran_id: u32) -> Vec<u8> {
    let packet = ConnectionResponse { action: 0, transaction_id: tran_id, connection_id: uuid};

    let encoded = bincode::encode(&packet, SizeLimit::Infinite).unwrap();
    encoded
}
