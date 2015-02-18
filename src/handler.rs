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
extern crate rand;
extern crate rusqlite;
extern crate "rustc-serialize" as rustc_serialize;
extern crate time;

use std::net::{IpAddr, SocketAddr, UdpSocket};

use self::rand::Rng;
use rusqlite::SqliteConnection;

use parse_packets::*;

// struct used by update announce to make passing data easy (vs. 4 more parameters)
struct ID {
    info_hash:  [u8; 20],
    ip:         u32,
    port:       u16,
    peer_id:    [u8; 20],
    remaining:  i64,
}

// Generate a UUID to make the client happy
fn gen_uuid() -> i64 {
    let mut rng = rand::thread_rng();
    let mut uuid = time::precise_time_ns();
    uuid <<= 32;
    uuid as i64 | rng.gen::<u32>() as i64
}

// On announce, update the client's remaining and last_active info
// Get the Seeders and Leechers for the provided info_hash
fn update_announce(conn: &SqliteConnection, id: &ID, data: &ClientAnnounce) -> (Vec<(i32,i32)>,i32, i32) {

    // Update the user info
    conn.execute(
        "INSERT OR REPLACE INTO torrent (info_hash, ip, port, peer_id, remaining, last_active)
        VALUES (?, ?, ?, ?, ?, strftime('%s', 'now'))",
        &[&id.info_hash.as_slice(), &(id.ip as i32), &(id.port as i32),
          &id.peer_id.as_slice(), &id.remaining]
    ).unwrap();

    // Info Hash swarm IP and ports
    // i32 due to current rusqlite type handling
    let mut swarm: Vec<(i32, i32)> = Vec::new();

    // Get Seeders
    let mut stmt = conn.prepare(
        "SELECT ip,port,COUNT(*)
         FROM torrent
         WHERE info_hash = ? AND remaining = 0
         GROUP BY ip,port"
    ).unwrap();

    // Each row produces a count, update it as we continue along
    let mut seeders: i32 = 0;
    for row in stmt.query(&[&data.info_hash.as_slice()]).unwrap().map(|row| row.unwrap()) {
        let i: i32 = row.get(0);
        let p: i32 = row.get(1);
        swarm.push((i,p));
        let c: i32 = row.get(2);
        if c > seeders {
            seeders = c;
        }
    }

    // Get Leechers
    let mut stmt = conn.prepare(
        "SELECT ip,port,COUNT(*)
         FROM torrent
         WHERE info_hash = ? AND remaining > 0
         GROUP BY ip,port"
    ).unwrap();

    // Each row produces a count, update it as we continue along
    let mut leechers: i32 = 0;
    for row in stmt.query(&[&data.info_hash.as_slice()]).unwrap().map(|row| row.unwrap()) {
        let i: i32 = row.get(0);
        let p: i32 = row.get(1);
        swarm.push((i,p));
        let c: i32 = row.get(2);
        if c > leechers {
            leechers = c;
        }
    }

    // Return the swarm, seeders, and leechers for packeting
    (swarm, seeders, leechers)
}

pub fn handle_response(tsock: UdpSocket, src: &SocketAddr, packet: Vec<u8>, conn: &SqliteConnection) {
    // Split the packet into header and body parts
    let mut packet_header = packet;
    let packet_body = packet_header.split_off(16);

    // parse the header to act on it
    let header = parse_header(&packet_header);

    match header.action {
        0 => {
            if header.connection_id == 0x41727101980 {
                // Cool Story, we got a new connection.
                // We need to generate an unique id for this client.
                // 32bits of the current time in nanoseconds combined with 32bits of
                // random numbers
                let uuid = gen_uuid();

                // Now they're in the db, let's say hi
                let encoded = encode_connect_response(uuid, header.transaction_id);
                tsock.send_to(&encoded, src).unwrap();
            } else {
                ()
            }
        },
        1 => {
            // Decode the announce info
            let decoded = decode_client_announce(&packet_body);

            // handle an IP of 0
            let mut ip = decoded.ip;
            if ip == 0 {
                ip = match src.ip() {
                    IpAddr::V4(x) => {
                        bincode::decode(&x.octets()).unwrap()
                    },
                    _ => panic!("This is possible?")
                };
            }

            // Package up the announce info for DB consumption
            let id = ID {
                info_hash: decoded.info_hash,
                ip: ip,
                port: decoded.port,
                peer_id: decoded.peer_id,
                remaining: decoded.remaining,
            };

            // Get the swarm, seeder, and leecher info
            let (swarm, seeders, leechers) = update_announce(conn, &id, &decoded);

            // Send it back to the client
            let serv_announce = encode_server_announce(
                header.transaction_id, swarm, leechers, seeders
            );
            tsock.send_to(&serv_announce, src).unwrap();
        },
        _ => {
            println!("Unhandled action: {:?}", header.action);
        },
    }
}
