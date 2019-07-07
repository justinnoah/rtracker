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

use r2d2;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{SQLITE_OPEN_READ_WRITE, SQLITE_OPEN_CREATE, SQLITE_OPEN_MEMORY,
               SQLITE_OPEN_FULL_MUTEX, SQLITE_OPEN_URI, SQLITE_OPEN_SHARED_CACHE};

pub type PoolCon = r2d2::PooledConnection<SqliteConnectionManager>;

pub fn db_connection_pool(pool_size: usize) -> r2d2::Pool<SqliteConnectionManager> {
    let flags = { SQLITE_OPEN_READ_WRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_MEMORY |
                  SQLITE_OPEN_FULL_MUTEX | SQLITE_OPEN_URI |
                  SQLITE_OPEN_SHARED_CACHE };
    debug!("{:?} threads available", pool_size);
    let config = r2d2::Config::builder()
        .pool_size(pool_size as u32).build();
    let manager = SqliteConnectionManager::new_with_flags(
        "file:blah?mode=memory&cache=shared", flags);
    r2d2::Pool::new(config, manager).unwrap()
}

// Initialize the database
pub fn db_init(conn: PoolCon) {
    conn.execute("
        CREATE TABLE IF NOT EXISTS torrent (
            info_hash   TEXT,
            ip          TEXT,
            port        INTEGER,
            peer_id     TEXT,
            remaining   INTEGER,
            last_active INTEGER,
            PRIMARY KEY (info_hash, ip, port, peer_id)
        );",
        &[]
    ).unwrap();
}

pub fn db_prune(conn: PoolCon) {
    conn.execute(
        "DELETE FROM torrent
        WHERE (strftime('%s','now') - last_active) > 1860;",
        &[]
    ).unwrap();
}
