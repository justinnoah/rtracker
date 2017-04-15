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
use bincode::Infinite;
use bincode::{deserialize, serialize, serialized_size};
use packet_data_types::*;


pub fn parse_header(packet: &[u8]) -> PacketHeader {
    // In case we send extra by mistake, make sure to only parse the first 16 bytes
    debug!("Deserializing header!");
    deserialize(&packet).unwrap()
}


pub fn encode_connect_response(uuid: i64, tran_id: i32) -> Vec<u8> {
    let packet = ConnectionResponse { action: 0, transaction_id: tran_id, connection_id: uuid};
    serialize(&packet, Infinite).unwrap()
}


pub fn decode_client_announce(packet: &[u8]) -> ClientAnnounce {
    let ca = ClientAnnounce::default();
    debug!("ClientAnnounce serialized size: {:?}", serialized_size(&ca));
    debug!("Deserializing Client Announce!");
    debug!("packet len : {:?}", packet.len());
    debug!("info_hash  : {:?}", &packet[..20]);
    debug!("peer_id    : {:?}", &packet[20..40]);
    debug!("downloaded : {:?}", &packet[40..48]);
    debug!("remaining  : {:?}", &packet[48..56]);
    debug!("uploaded   : {:?}", &packet[56..64]);
    debug!("event      : {:?}", &packet[64..68]);
    debug!("ip         : {:?}", &packet[68..72]);
    debug!("key        : {:?}", &packet[72..76]);
    debug!("num_want   : {:?}", &packet[76..80]);
    debug!("port       : {:?}", &packet[80..82]);
    if packet.len() > 82 {
        debug!("extensions : {:?}", &packet[82..]);
    }

    match deserialize(&packet) {
        Ok(x) => x,
        Err(p) => {
            error!("{:?}", p);
            panic!("{:?}", p)
        },
    }
}


pub fn encode_server_announce(transaction_id: i32, mut swarm: Vec<(i32,i32)>, num_want: i32, leechers: i32, seeders: i32) -> Vec<u8> {
    let packet = ServerAnnounce {
        // Announce is always 1
        action:         1,
        transaction_id: transaction_id,
        // 30min in secs
        interval:       1800,
        leechers:       leechers,
        seeders:        seeders,
    };

    let mut packet = serialize(&packet, Infinite).unwrap();

    // Truncate the vector if num_want is smaller than the vector length
    if (num_want >= 0) && (num_want < swarm.len() as i32) {
        swarm.truncate(num_want as usize);
    }

    for peer in &mut swarm {
        let (i, p): (i32, i32) = *peer;
        packet.append(&mut serialize(&i, Infinite).unwrap());
        packet.append(&mut serialize(&(p as u16), Infinite).unwrap());
    }

    packet
}


pub fn encode_error(transaction_id: i32, error_string: &'static str) -> Vec<u8> {
    let mut packet: Vec<u8> = Vec::new();

    // Action (3 == Error)
    packet.append(&mut serialize(&3i32, Infinite).unwrap());
    // Transaction_id
    packet.append(&mut serialize(&transaction_id, Infinite).unwrap());
    // Finally, the message
    packet.append(&mut serialize(&error_string, Infinite).unwrap());

    // Return the packet
    packet
}
