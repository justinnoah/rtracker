//  rtracker: bittorrent tracker
//  Copyright (C) 2019  Justin Noah <justinnoah@gmail.com>
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.
//
//  This program is distributed in the hope that it will be useful,
//   but WITHOUT ANY WARRANTY; without even the implied warranty of
//   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//   GNU Affero General Public License for more details.
//
//   You should have received a copy of the GNU Affero General Public License
//   along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Reminder:
//
// NetworkEndian = Big Endian

use std::net::IpAddr;
use std::str::FromStr;

use bincode::{config as bin, serialized_size};

use packet_data_types::*;

pub fn parse_header(packet: &[u8]) -> PacketHeader {
    // In case we send extra by mistake, make sure to only parse the first 16 bytes
    debug!("Deserializing header of len {:?}", packet.len());

    let b: i64 = bin()
        .big_endian()
        .deserialize(&Vec::from(&packet[0..8]))
        .unwrap();
    debug!("ID Bytes: {:?}", &packet[0..8]);
    debug!("ID: {0:x}", b);

    let c: i32 = bin()
        .big_endian()
        .deserialize(&Vec::from(&packet[8..12]))
        .unwrap();
    debug!("Action Bytes: {:?}", &packet[8..12]);
    debug!("Action: {:?}", c);

    let d: i32 = bin()
        .big_endian()
        .deserialize(&Vec::from(&packet[12..]))
        .unwrap();
    debug!("TID Bytes: {:?}", &packet[12..]);
    debug!("TID: {:?}", d);

    bin()
        .big_endian()
        .deserialize::<PacketHeader>(&packet)
        .unwrap()
}

pub fn encode_server_connect(uuid: i64, tran_id: i32) -> Vec<u8> {
    let packet = ConnectionResponse {
        action: 0,
        transaction_id: tran_id,
        connection_id: uuid,
    };

    // Network Order, Bounded(16)
    let v: Vec<u8> = bin().big_endian().limit(16).serialize(&packet).unwrap();

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

    match bin().big_endian().deserialize(&packet) {
        Ok(x) => x,
        Err(p) => panic!("{:?}", p),
    }
}

pub fn encode_server_announce(
    transaction_id: i32,
    mut swarm: Vec<(String, i32)>,
    num_want: i32,
    leechers: i32,
    seeders: i32,
) -> Vec<u8> {
    let packet = ServerAnnounce {
        // Action for Announce is always 1
        action: 1,
        transaction_id,
        // 30min in secs
        interval: 1800,
        leechers,
        seeders,
    };

    let mut packet = bin().big_endian().serialize(&packet).unwrap();

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
            }
            IpAddr::V6(ip6) => {
                let double_bytes = ip6.segments();
                options()
                    .with_big_endian()
                    .with_limit(16)
                    .serialize(&double_bytes)
                    .unwrap()
            }
        };

        packet.append(&mut ip_bytes);
        packet.append(&mut bin().big_endian().limit(2).serialize(&(p as u16)).unwrap());
    }

    packet
}

pub fn encode_error(transaction_id: i32, error_string: &str) -> Vec<u8> {
    let err = ServerError {
        // Action (3 == Error)
        action: 3,
        transaction_id,
        error: String::from_str(error_string).unwrap(),
    };
    debug!("{:?}", err);

    // Return the packet
    bin().big_endian().serialize(error_string).unwrap()
}
