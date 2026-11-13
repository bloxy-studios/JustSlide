//! Development-only deterministic design fixture (Figma reference content).
//! Compiled only with `test-engine`/`design-fixture`; never in release apps.

use crate::ids::{new_id, now_ms};
use crate::model::*;
use crate::state::State;
use serde_json::Value;

pub const DESIGN_FIXTURE: &str = include_str!("../../../tests/fixtures/design/slide-fixture.json");

pub fn apply(s: &mut State, fx: &Value) -> anyhow::Result<()> {
    let variant = fx["variant"].as_str().unwrap_or("desktop");
    let base: Value = serde_json::from_str(DESIGN_FIXTURE)?;
    let fx = if fx.get("tabs").is_some() { fx } else { &base };
    let personal = s.personal_workspace_id().map(str::to_string).ok_or_else(|| anyhow::anyhow!("no personal workspace"))?;
    s.tabs.clear();
    s.favorites.clear();
    for (i, f) in fx["favorites"].as_array().into_iter().flatten().enumerate() {
        s.favorites.push(Favorite { id: new_id("fav"), url: f["url"].as_str().unwrap_or_default().into(), title: f["title"].as_str().unwrap_or_default().into(), order: i as i64 });
    }
    let push = |s: &mut State, item: &Value, pinned: bool, order: i64| -> String {
        let mut t = s.new_tab(&personal, item["url"].as_str().unwrap_or_default(), pinned, OpenedBy::Restore, None);
        t.title = item["title"].as_str().unwrap_or_default().into();
        t.favicon_url = item["icon"].as_str().map(|k| format!("fixture:{k}"));
        t.load_state = LoadState::Loaded;
        t.order = order;
        t.created_at = now_ms();
        t.secure = true;
        let id = t.id.clone();
        s.tabs.push(t);
        id
    };
    for (i, p) in fx["pinned"].as_array().into_iter().flatten().enumerate() {
        push(s, p, true, i as i64);
    }
    for (i, t) in fx["tabs"].as_array().into_iter().flatten().enumerate() {
        let id = push(s, t, false, i as i64);
        if t["active"].as_bool() == Some(true) {
            s.window.active_workspace_id = personal.clone();
            s.window.active_tab_by_workspace.insert(personal.clone(), id);
        }
    }
    let v = &base["variants"][variant];
    if let Some(w) = s.workspace_mut(&personal) {
        w.pinned_collapsed = v["pinnedCollapsed"].as_bool().unwrap_or(false);
    }
    Ok(())
}
