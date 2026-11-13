# Product requirements (summary)

Users: people who browse all day and also delegate browsing tasks to AI agents (Claude Code, Codex, ChatGPT, Antigravity, Cursor).
Differentiator: every supported browser operation is a permission-controlled command available to the person and to authorised agents — not a chatbot sidebar. Agents work in their own workspaces without moving the person's focus.
MVP: native macOS WebKit browser with the Figma sidebar (favorites, pinned, tabs, workspace dock), persistent workspaces/tabs/favorites, local MCP with scoped grants, pause/revoke, visible agent activity, screenshots of the requested tab.
Non-goals (v1): Chromium/Chrome-extension parity, cloud relay, built-in LLM, Windows/Linux.
Acceptance: spec §20 tests 1–25; current status per test in `docs/progress.md`.
