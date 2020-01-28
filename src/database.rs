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
use rusqlite::*;

pub type PoolCon = r2d2::PooledConnection<SqliteConnectionManager>;

pub fn db_connection_pool(pool_size: usize) -> r2d2::Pool<SqliteConnectionManager> {
    let flags = {
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_MEMORY
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_SHARED_CACHE
    };

    debug!("{:?} threads available", pool_size);

    let manager =
        SqliteConnectionManager::file("file:blah?mode=memory&cache=shared").with_flags(flags);

    r2d2::Pool::builder()
        .max_size(pool_size as u32)
        .build(manager)
        .unwrap()
}

// Initialize the database
pub fn db_init(conn: PoolCon) {
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS torrent (
            info_hash   TEXT,
            ip          TEXT,
            port        INTEGER,
            peer_id     TEXT,
            remaining   INTEGER,
            last_active INTEGER,
            PRIMARY KEY (info_hash, ip, port, peer_id)
        );",
        params![],
    )
    .unwrap();
}

pub fn db_prune(conn: PoolCon) {
    conn.execute(
        "DELETE FROM torrent
        WHERE (strftime('%s','now') - last_active) > 1860;",
        params![],
    )
    .unwrap();
}
