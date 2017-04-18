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

use std::str::FromStr;
use std::net::IpAddr;

use bincode::{Bounded, Infinite, serialized_size};
use bincode::endian_choice::{deserialize, serialize};
use byteorder::NetworkEndian;

use packet_data_types::*;


pub fn parse_header(packet: &[u8]) -> PacketHeader {
    // In case we send extra by mistake, make sure to only parse the first 16 bytes
    debug!("Deserializing header of len {:?}", packet.len());
    let b :i64 = deserialize::<i64, NetworkEndian>(&Vec::from(&packet[0..8])).unwrap();
    debug!("ID Bytes: {:?}", &packet[0..8]);
    debug!("ID: {0:x}", b);
    let c :i32 = deserialize::<i32, NetworkEndian>(&Vec::from(&packet[8..12])).unwrap();
    debug!("Action Bytes: {:?}", &packet[8..12]);
    debug!("Action: {:?}", c);
    let d :i32 = deserialize::<i32, NetworkEndian>(&Vec::from(&packet[12..])).unwrap();
    debug!("TID Bytes: {:?}", &packet[12..]);
    debug!("TID: {:?}", d);
    deserialize::<PacketHeader, NetworkEndian>(&packet).unwrap()
}


pub fn encode_server_connect(uuid: i64, tran_id: i32) -> Vec<u8> {
    let packet = ConnectionResponse { action: 0, transaction_id: tran_id, connection_id: uuid};
    let v: Vec<u8> = serialize::<_,_,NetworkEndian>(&packet, Bounded(16)).unwrap();
    debug!("v: {:?}", v);
    v
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

    match deserialize::<ClientAnnounce, NetworkEndian>(&packet) {
        Ok(x) => x,
        Err(p) => {
            panic!("{:?}", p)
        },
    }
}


pub fn encode_server_announce(transaction_id: i32, mut swarm: Vec<(String,i32)>, num_want: i32, leechers: i32, seeders: i32) -> Vec<u8> {
    let packet = ServerAnnounce {
        // Announce is always 1
        action:         1,
        transaction_id: transaction_id,
        // 30min in secs
        interval:       1800,
        leechers:       leechers,
        seeders:        seeders,
    };

    let mut packet = serialize::<_,_,NetworkEndian>(&packet, Infinite).unwrap();

    // Truncate the vector if num_want is smaller than the vector length
    if (num_want >= 0) && (num_want < swarm.len() as i32) {
        swarm.truncate(num_want as usize);
    }

    for peer in swarm {
        let (i, p): (String, i32) = peer;
        let ip = IpAddr::from_str(&i).unwrap();
        let mut ip_bytes = match ip {
            IpAddr::V4(ip4) => {
                let bytes = ip4.octets();
                let mut it: Vec<u8> = Vec::new();
                it.extend_from_slice(&bytes);
                it
            },
            IpAddr::V6(ip6) => {
                let double_bytes = ip6.segments();
                let it = serialize::<_,_,NetworkEndian>(&double_bytes, Bounded(16)).unwrap();
                it
            },
        };

        packet.append(&mut ip_bytes);
        packet.append(&mut serialize::<_,_,NetworkEndian>(&(p as u16), Bounded(2)).unwrap());
    }

    packet
}


pub fn encode_error(transaction_id: i32, error_string: &str) -> Vec<u8> {
    let err = ServerError {
        // Action (3 == Error)
        action: 3,
        transaction_id: transaction_id,
        error: String::from_str(error_string).unwrap(),
    };

    debug!("{:?}", err);
    // Return the packet
    serialize::<_,_,NetworkEndian>(error_string, Infinite).unwrap()
}
