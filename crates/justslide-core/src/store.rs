//! Transactional SQLite persistence for app-owned metadata. WebKit website
//! data (cookies, storage, credentials) is never copied here; it lives in
//! per-profile `WKWebsiteDataStore`s.

use crate::model::*;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

/// Ordered, append-only migrations. `PRAGMA user_version` = applied count.
const MIGRATIONS: &[&str] = &[
    // 1: initial schema
    "CREATE TABLE profiles(id TEXT PRIMARY KEY, kind TEXT NOT NULL, name TEXT NOT NULL, data_store_id TEXT NOT NULL UNIQUE);
     CREATE TABLE workspaces(id TEXT PRIMARY KEY, name TEXT NOT NULL, icon TEXT NOT NULL, profile_id TEXT NOT NULL REFERENCES profiles(id),
        agent_owned INTEGER NOT NULL, owner_grant_id TEXT, ord INTEGER NOT NULL, pinned_collapsed INTEGER NOT NULL DEFAULT 0);
     CREATE TABLE tabs(id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE, url TEXT NOT NULL,
        title TEXT NOT NULL, favicon_url TEXT, pinned INTEGER NOT NULL, ord INTEGER NOT NULL, opened_by TEXT NOT NULL, opened_by_grant TEXT, created_at INTEGER NOT NULL);
     CREATE TABLE favorites(id TEXT PRIMARY KEY, url TEXT NOT NULL, title TEXT NOT NULL, ord INTEGER NOT NULL);
     CREATE TABLE kv(k TEXT PRIMARY KEY, v TEXT NOT NULL);
     CREATE TABLE grants(id TEXT PRIMARY KEY, client TEXT NOT NULL, label TEXT NOT NULL, token_hash TEXT NOT NULL UNIQUE, operations TEXT NOT NULL,
        workspace_ids TEXT NOT NULL, created_at INTEGER NOT NULL, expires_at INTEGER, revoked INTEGER NOT NULL, paused INTEGER NOT NULL);
     CREATE TABLE activity(seq INTEGER PRIMARY KEY AUTOINCREMENT, at INTEGER NOT NULL, grant_id TEXT NOT NULL, client TEXT NOT NULL, command TEXT NOT NULL,
        outcome TEXT NOT NULL, workspace_id TEXT, tab_id TEXT, url TEXT);
     CREATE TABLE closed_tabs(seq INTEGER PRIMARY KEY AUTOINCREMENT, workspace_id TEXT NOT NULL, url TEXT NOT NULL, title TEXT NOT NULL, pinned INTEGER NOT NULL, closed_at INTEGER NOT NULL);
     CREATE TABLE history(seq INTEGER PRIMARY KEY AUTOINCREMENT, profile_id TEXT NOT NULL, url TEXT NOT NULL, title TEXT NOT NULL, at INTEGER NOT NULL);
     CREATE INDEX history_url ON history(url);",
];

pub const ACTIVITY_RETENTION: i64 = 2000;
pub const CLOSED_TAB_RETENTION: i64 = 50;
pub const HISTORY_RETENTION: i64 = 50_000;

pub struct Store {
    conn: Connection,
}

#[derive(Default)]
pub struct Loaded {
    pub profiles: Vec<Profile>,
    pub workspaces: Vec<Workspace>,
    pub tabs: Vec<Tab>,
    pub favorites: Vec<Favorite>,
    pub grants: Vec<Grant>,
    pub settings: Option<Settings>,
    pub window: Option<WindowState>,
    pub instance_seed: Option<String>,
}

impl Store {
    pub fn open(path: &Path) -> rusqlite::Result<Store> {
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    pub fn in_memory() -> rusqlite::Result<Store> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> rusqlite::Result<Store> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        let mut s = Store { conn };
        s.migrate()?;
        Ok(s)
    }

    pub fn schema_version(&self) -> rusqlite::Result<i64> {
        self.conn.pragma_query_value(None, "user_version", |r| r.get(0))
    }

    fn migrate(&mut self) -> rusqlite::Result<()> {
        let current = self.schema_version()? as usize;
        if current > MIGRATIONS.len() {
            // A newer build wrote this database; refuse rather than corrupt it.
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "database schema {current} is newer than this build ({})",
                MIGRATIONS.len()
            )));
        }
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(current) {
            let tx = self.conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", (i + 1) as i64)?;
            tx.commit()?;
        }
        Ok(())
    }

    pub fn load(&self) -> rusqlite::Result<Loaded> {
        let c = &self.conn;
        let mut out = Loaded::default();
        let mut st = c.prepare("SELECT id, kind, name, data_store_id FROM profiles")?;
        out.profiles = st
            .query_map([], |r| {
                Ok(Profile { id: r.get(0)?, kind: ProfileKind::parse(&r.get::<_, String>(1)?).unwrap_or(ProfileKind::Personal), name: r.get(2)?, data_store_id: r.get(3)? })
            })?
            .collect::<Result<_, _>>()?;
        let mut st = c.prepare("SELECT w.id, w.name, w.icon, w.profile_id, p.kind, w.agent_owned, w.owner_grant_id, w.ord, w.pinned_collapsed FROM workspaces w JOIN profiles p ON p.id=w.profile_id ORDER BY w.ord")?;
        out.workspaces = st
            .query_map([], |r| {
                Ok(Workspace {
                    id: r.get(0)?, name: r.get(1)?, icon: r.get(2)?, profile_id: r.get(3)?,
                    profile_kind: ProfileKind::parse(&r.get::<_, String>(4)?).unwrap_or(ProfileKind::Personal),
                    agent_owned: r.get(5)?, owner_grant_id: r.get(6)?, order: r.get(7)?, pinned_collapsed: r.get(8)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        let mut st = c.prepare("SELECT id, workspace_id, url, title, favicon_url, pinned, ord, opened_by_grant, created_at FROM tabs ORDER BY ord")?;
        out.tabs = st
            .query_map([], |r| {
                Ok(Tab {
                    id: r.get(0)?, workspace_id: r.get(1)?, url: r.get(2)?, title: r.get(3)?, favicon_url: r.get(4)?, pinned: r.get(5)?, order: r.get(6)?,
                    load_state: LoadState::NotLoaded, error: None, opened_by: OpenedBy::Restore, opened_by_grant: r.get(7)?,
                    navigation_id: String::new(), can_go_back: false, can_go_forward: false, audible: false, secure: false, created_at: r.get(8)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        let mut st = c.prepare("SELECT id, url, title, ord FROM favorites ORDER BY ord")?;
        out.favorites = st.query_map([], |r| Ok(Favorite { id: r.get(0)?, url: r.get(1)?, title: r.get(2)?, order: r.get(3)? }))?.collect::<Result<_, _>>()?;
        let mut st = c.prepare("SELECT id, client, label, token_hash, operations, workspace_ids, created_at, expires_at, revoked, paused FROM grants")?;
        out.grants = st
            .query_map([], |r| {
                Ok(Grant {
                    id: r.get(0)?, client: r.get(1)?, label: r.get(2)?, token_hash: r.get(3)?,
                    operations: serde_json::from_str(&r.get::<_, String>(4)?).unwrap_or_default(),
                    workspace_ids: serde_json::from_str(&r.get::<_, String>(5)?).unwrap_or_default(),
                    created_at: r.get(6)?, expires_at: r.get(7)?, revoked: r.get(8)?, paused: r.get(9)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        out.settings = self.kv("settings")?.and_then(|s| serde_json::from_str(&s).ok());
        out.window = self.kv("window")?.and_then(|s| serde_json::from_str(&s).ok());
        out.instance_seed = self.kv("instance")?;
        Ok(out)
    }

    fn kv(&self, k: &str) -> rusqlite::Result<Option<String>> {
        self.conn.query_row("SELECT v FROM kv WHERE k=?1", [k], |r| r.get(0)).optional()
    }

    /// Persist the full app-owned snapshot in one transaction. Private
    /// profiles are excluded. Small tables; a full rewrite keeps the write
    /// path obviously correct (see docs/performance-engineering.md).
    pub fn save(&mut self, s: &crate::state::State) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM tabs", [])?;
        tx.execute("DELETE FROM workspaces", [])?;
        tx.execute("DELETE FROM profiles", [])?;
        tx.execute("DELETE FROM favorites", [])?;
        for p in s.profiles.values().filter(|p| p.kind.persists()) {
            tx.execute("INSERT INTO profiles VALUES(?1,?2,?3,?4)", params![p.id, p.kind.as_str(), p.name, p.data_store_id])?;
        }
        for w in s.workspaces.iter().filter(|w| w.profile_kind.persists()) {
            tx.execute(
                "INSERT INTO workspaces VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                params![w.id, w.name, w.icon, w.profile_id, w.agent_owned, w.owner_grant_id, w.order, w.pinned_collapsed],
            )?;
        }
        for t in s.tabs.iter().filter(|t| s.tab_persists(t)) {
            tx.execute(
                "INSERT INTO tabs VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![t.id, t.workspace_id, t.url, t.title, t.favicon_url, t.pinned, t.order, t.opened_by.as_str(), t.opened_by_grant, t.created_at],
            )?;
        }
        for f in &s.favorites {
            tx.execute("INSERT INTO favorites VALUES(?1,?2,?3,?4)", params![f.id, f.url, f.title, f.order])?;
        }
        tx.execute("INSERT OR REPLACE INTO kv VALUES('settings', ?1)", [serde_json::to_string(&s.settings).unwrap_or_default()])?;
        let mut w = s.window.clone();
        w.active_tab_by_workspace.retain(|ws, _| s.workspaces.iter().any(|x| &x.id == ws && x.profile_kind.persists()));
        tx.execute("INSERT OR REPLACE INTO kv VALUES('window', ?1)", [serde_json::to_string(&w).unwrap_or_default()])?;
        tx.execute("INSERT OR REPLACE INTO kv VALUES('instance', ?1)", [&s.instance_id])?;
        tx.commit()
    }

    pub fn save_grant(&mut self, g: &Grant) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO grants VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![g.id, g.client, g.label, g.token_hash, serde_json::to_string(&g.operations).unwrap(), serde_json::to_string(&g.workspace_ids).unwrap(), g.created_at, g.expires_at, g.revoked, g.paused],
        )?;
        Ok(())
    }

    pub fn append_activity(&mut self, a: &ActivityRecord) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO activity(at, grant_id, client, command, outcome, workspace_id, tab_id, url) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![a.at, a.grant_id, a.client, a.command, a.outcome, a.workspace_id, a.tab_id, a.url],
        )?;
        tx.execute("DELETE FROM activity WHERE seq <= (SELECT MAX(seq) FROM activity) - ?1", [ACTIVITY_RETENTION])?;
        tx.commit()
    }

    pub fn activity(&self, limit: i64) -> rusqlite::Result<Vec<ActivityRecord>> {
        let mut st = self.conn.prepare("SELECT at, grant_id, client, command, outcome, workspace_id, tab_id, url FROM activity ORDER BY seq DESC LIMIT ?1")?;
        st.query_map([limit], |r| {
            Ok(ActivityRecord { at: r.get(0)?, grant_id: r.get(1)?, client: r.get(2)?, command: r.get(3)?, outcome: r.get(4)?, workspace_id: r.get(5)?, tab_id: r.get(6)?, url: r.get(7)? })
        })?
        .collect()
    }

    pub fn push_closed(&mut self, c: &ClosedTab) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("INSERT INTO closed_tabs(workspace_id, url, title, pinned, closed_at) VALUES(?1,?2,?3,?4,?5)", params![c.workspace_id, c.url, c.title, c.pinned, c.closed_at])?;
        tx.execute("DELETE FROM closed_tabs WHERE seq <= (SELECT MAX(seq) FROM closed_tabs) - ?1", [CLOSED_TAB_RETENTION])?;
        tx.commit()
    }

    pub fn pop_closed(&mut self, workspace_id: &str) -> rusqlite::Result<Option<ClosedTab>> {
        let tx = self.conn.transaction()?;
        let row = tx
            .query_row(
                "SELECT seq, workspace_id, url, title, pinned, closed_at FROM closed_tabs WHERE workspace_id=?1 ORDER BY seq DESC LIMIT 1",
                [workspace_id],
                |r| Ok((r.get::<_, i64>(0)?, ClosedTab { workspace_id: r.get(1)?, url: r.get(2)?, title: r.get(3)?, pinned: r.get(4)?, closed_at: r.get(5)? })),
            )
            .optional()?;
        if let Some((seq, _)) = &row {
            tx.execute("DELETE FROM closed_tabs WHERE seq=?1", [seq])?;
        }
        tx.commit()?;
        Ok(row.map(|(_, c)| c))
    }

    pub fn add_history(&mut self, profile_id: &str, url: &str, title: &str, at: i64) -> rusqlite::Result<()> {
        self.conn.execute("INSERT INTO history(profile_id, url, title, at) VALUES(?1,?2,?3,?4)", params![profile_id, url, title, at])?;
        Ok(())
    }

    pub fn search_history(&self, profile_id: &str, q: &str, limit: i64) -> rusqlite::Result<Vec<HistoryEntry>> {
        let like = format!("%{}%", q.replace('%', "\\%").replace('_', "\\_"));
        let mut st = self.conn.prepare(
            "SELECT url, title, MAX(at) FROM history WHERE profile_id=?1 AND (url LIKE ?2 ESCAPE '\\' OR title LIKE ?2 ESCAPE '\\') GROUP BY url ORDER BY MAX(at) DESC LIMIT ?3",
        )?;
        st.query_map(params![profile_id, like, limit], |r| Ok(HistoryEntry { url: r.get(0)?, title: r.get(1)?, at: r.get(2)? }))?.collect()
    }
}
