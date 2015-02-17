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
use rustc_serialize::{Decodable, Decoder};


#[derive(RustcDecodable)]
pub struct PacketHeader {
    pub connection_id:  i64,
    pub action:         i32,
    pub transaction_id: i32,
}

#[derive(Debug, RustcEncodable)]
struct ConnectionResponse {
    action:         i32,
    transaction_id: i32,
    connection_id:  i64,
}

#[derive(Debug)]
struct ClientAnnounce {
    info_hash:  [u8; 20],
    peer_id:    [u8; 20],
    downloaded: i64,
    remaining:  i64,
    uploaded:   i64,
    event:      i32,
    ip:         u32,
    key:        u32,
    num_want:   i32,
    port:       u16,
}

impl Decodable for ClientAnnounce {
    fn decode<D: Decoder>(d: &mut D) -> Result<ClientAnnounce, D::Error> {
        let mut info_hash = [0u8; 20];
        for i in 0..20 {
            info_hash[i] = try!(d.read_u8());
        }
        let mut peer_id = [0u8; 20];
        for i in 0..20 {
            peer_id[i] = try!(d.read_u8());
        }
        Ok(ClientAnnounce {
            info_hash:  info_hash,
            peer_id:    peer_id,
            downloaded: try!(d.read_i64()),
            remaining:  try!(d.read_i64()),
            uploaded:   try!(d.read_i64()),
            event:      try!(d.read_i32()),
            ip:         try!(d.read_u32()),
            key:        try!(d.read_u32()),
            num_want:   try!(d.read_i32()),
            port:       try!(d.read_u16()),
        })
    }
}

#[derive(Debug, RustcEncodable)]
struct ServerAnnounce {
    action:         i32,
    transaction_id: i32,
    interval:       i32,
    leechers:       i32,
    seeders:        i32,
}

pub fn parse_header(packet: &[u8]) -> PacketHeader {
    // In case we send extra by mistake, make sure to only parse the first 16 bytes
    bincode::decode(&packet[..16]).unwrap()
}

pub fn encode_connect_response(uuid: i64, tran_id: i32) -> Vec<u8> {
    let packet = ConnectionResponse { action: 0, transaction_id: tran_id, connection_id: uuid};
    bincode::encode(&packet, SizeLimit::Infinite).unwrap()
}

pub fn decode_client_announce(packet: &[u8]) -> ClientAnnounce {
    bincode::decode(&packet).unwrap()
}
