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
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use bincode::{Bounded, serialize};
use chrono::UTC;
use rand::{Rng, thread_rng};

use database::db_connect;
use packet_data_types::*;
use parse_packets::*;


// struct used by update announce to make passing data easy (vs. 4 more parameters)
struct ID {
    info_hash:  Vec<u8>,
    ip:         String,
    port:       u16,
    peer_id:    Vec<u8>,
    remaining:  i64,
}

pub type TrackerData = (Vec<(String,i32)>, i32, i32);

// Generate a UUID to make the client happy
fn gen_uuid() -> i64 {
    let mut rng = thread_rng();
    let mut uuid: i64 = UTC::now().timestamp();
    uuid <<= 32;
    uuid | rng.gen::<u32>() as i64
}

// On announce, update the client's remaining and last_active info
// Get the Seeders and Leechers for the provided info_hash
fn update_announce(path: String, id: &ID, data: &ClientAnnounce) -> TrackerData {
    // [u8; 20] -> Vec<u8>
    let mut hash: Vec<u8> = Vec::new();
    hash.extend_from_slice(&data.info_hash);
    debug!("ClientAnnounce");
    debug!("hash: {:?}", hash);

    let conn = db_connect(&path);

    // Update the user info
    match conn.execute(
        "INSERT OR REPLACE INTO torrent (info_hash, ip, port, peer_id, remaining, last_active)
        VALUES (?, ?, ?, ?, ?, strftime('%s', 'now'))",
        &[&id.info_hash, &(id.ip.to_string()), &(id.port as i32),
          &id.peer_id, &id.remaining]
    ) {
        Ok(_) => (),
        Err(x) => panic!("{:?}", x)
    }

    // Info Hash swarm IP and ports
    // i32 due to current rusqlite type handling
    let mut swarm: Vec<(String, i32)> = Vec::new();

    // Get Seeders
    let mut stmt = conn.prepare(
        "SELECT ip,port,COUNT(*)
         FROM torrent
         WHERE info_hash = ? AND remaining = 0
         GROUP BY ip,port"
    ).unwrap();
    let mut rows = stmt.query(&[&hash]).unwrap();

    // Each row produces a count, update it as we continue along
    let mut seeders: i32 = 0;
    while let Some(result_row) = rows.next() {
        let row = result_row.unwrap();
        let ip: String = row.get(0);
        let port: i32 = row.get(1);
        swarm.push((ip, port));
        let count: i32 = row.get(2);
        if count > seeders {
            seeders = count;
        }
    }

    // Get Leechers
    let mut stmt = conn.prepare(
        "SELECT ip,port,COUNT(*)
         FROM torrent
         WHERE info_hash = ? AND remaining > 0
         GROUP BY ip,port"
    ).unwrap();
    let mut rows = stmt.query(&[&hash]).unwrap();

    // Each row produces a count, update it as we continue along
    let mut leechers: i32 = 0;
    while let Some(result_row) = rows.next() {
        let row = result_row.unwrap();
        let ip: String = row.get(0);
        let port: i32 = row.get(1);
        swarm.push((ip, port));
        let count: i32 = row.get(2);
        if count > leechers {
            leechers = count;
        }
    }

    // Return the swarm, seeders, and leechers for packeting
    (swarm, seeders, leechers)
}

pub fn handle_received_packet(packet: Vec<u8>, src: SocketAddr, sock: UdpSocket, db_path: &String) {
    debug!("Begin parsing received packet!");
    let (packet_header, packet_body) = packet.split_at(16);
    debug!("Packet Size: {:?}", packet.len());

    // parse the header to act on it
    let header :PacketHeader = parse_header(&packet_header);
    debug!("Header: {:?}", header);
    debug!("Action: {}", header.action as i32);
    debug!("Packet Body (PB):");
    debug!("(PB) Length: {}", packet_body.len());
    match header.action {
        0 => {
            debug!("Conid? {}", header.connection_id);
            // Magic number according to
            // http://www.rasterbar.com/products/libtorrent/udp_tracker_protocol.html
            // if header.connection_id == 0x41727101980 {
                // We need to generate an unique id for this client.
                // 32bits of the current time in nanoseconds combined with 32bits of
                // random numbers
                let uuid = gen_uuid();

                // debugs
                debug!("UUID: {}", uuid);

                // Now they're in the db, let's say hi
                let encoded = encode_server_connect(uuid, header.transaction_id);
                sock.send_to(&encoded, src).unwrap();
            //} else {
            //}

        },
        1 => {
            // Decode the announce info
            let ca_decoded: ClientAnnounce = decode_client_announce(&packet_body);

            // handle an IP of 0
            let ip_field = ca_decoded.ip;
            let mut ip = String::new();
            if ip_field == 0 {
                ip = match src.ip() {
                    IpAddr::V4(x) => {
                        x.to_string()
                    },
                    IpAddr::V6(y) => {
                        y.to_string()
                    }
                };
            } else {
                // This is guaranteed to be a u32 and thus have a Vec<u8>.len() of 4
                let x :Vec<u8> = serialize(&ca_decoded.ip, Bounded(4)).unwrap();
                ip = Ipv4Addr::new(x[0], x[1], x[2], x[3]).to_string();
            }


            // Package up the announce info for DB consumption
            let mut hash: Vec<u8> = Vec::with_capacity(20);
            hash.extend_from_slice(&ca_decoded.info_hash);
            let mut peer_id: Vec<u8> = Vec::with_capacity(20);
            peer_id.extend_from_slice(&ca_decoded.peer_id);
            let id = ID {
                info_hash: hash,
                ip: ip,
                port: ca_decoded.port,
                peer_id: peer_id,
                remaining: ca_decoded.remaining,
            };

            // Get the swarm, seeder, and leecher info
            let (swarm, seeders, leechers) = update_announce(db_path.clone(), &id, &ca_decoded);

            // Send it back to the client
            let serv_announce = encode_server_announce(
                header.transaction_id, swarm, ca_decoded.num_want, leechers, seeders
            );
            sock.send_to(&serv_announce, src).unwrap();
        },
        _ => {
            let err_packet = encode_error(header.transaction_id, "Unsupported Action");
            sock.send_to(&err_packet, src).unwrap();
        }
    }
}
