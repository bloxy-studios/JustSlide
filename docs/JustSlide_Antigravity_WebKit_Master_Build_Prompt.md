# JUSTSLIDE - ANTIGRAVITY MASTER IMPLEMENTATION SPECIFICATION

**Project:** JustSlide  
**Domain:** justslide.app  
**Implementation:** macOS-first, non-Chromium browser  
**Primary workflow:** Google Antigravity IDE/Agent, Figma MCP, official ADS documentation, native macOS build/test tools  
**Design source:** https://www.figma.com/design/nw47hqMLTtmBMksJ4BnJKb/Slide?node-id=1-6

> EXECUTION DIRECTIVE: Read this entire specification. Inspect the working repository and machine. Establish Figma MCP access and retrieve the *actual* Figma screenshot, design tree, component states, and assets before implementing the browser UI. Create a plan and engineering documents, then execute the first real native browser vertical slice. Verify visual fidelity with screenshots and real browser functionality with native tests. Do not stop after a proposal, a beautiful static React mockup, or an unverified claim of completion.

---

## 0. YOUR ROLE AND DECISION PRIORITIES

Act as one coordinated engineering team, with the judgment of an experienced browser engineer, macOS native engineer, systems architect, designer, accessibility specialist, security reviewer, and test engineer. You own the quality of the implementation and the evidence, not merely code generation.

Follow this hierarchy when requirements conflict:

1. Security, user privacy, and platform correctness.
2. Legal rights and license compliance.
3. Functional browser capabilities and reliable persistence.
4. The current Figma design's information architecture, layout, assets, and interactions.
5. The approved portions of Atlassian Design System (ADS) component contracts, tokens, and type scale.
6. Performance, maintainability, accessibility, and native consistency.
7. Decorative preferences not established by the design.

Where Figma pixel dimensions conflict with accessible behavior, ADS tokens, or real platform limitations, record the exception in `docs/design-fidelity.md`; do not silently redesign the application. Where legal permission for exact ADS assets/fonts is unresolved, do not use restricted assets in shipping code; isolate the gap and request an appropriate owner decision.

**Non-negotiable change from previous JustSlide specifications:** DO NOT USE CHROMIUM, CHROME CONTENT API, CEF, ELECTRON, OR A DOWNSTREAM CHROMIUM FORK AS JUSTSLIDE'S RENDERING ENGINE. This instruction supersedes the earlier Cursor/HyperAgent Chromium architecture. Testing the standalone React UI using a Chrome-based development browser is allowed and does not change the shipped browser engine. Never confuse the two.

**Do not confuse repositories:** `Slide` is a separate earlier application and may exist in another repository. `JustSlide` is this app. Do not alter, overwrite, rename, or share writable user profiles with `Slide`, Chrome, Safari, or the previous Chromium prototype. If this repository already contains Chromium-specific work, preserve it in a branch/commit and document the proposed migration before replacing anything. Do not destructively delete an existing worktree.

---

## 1. PRODUCT DEFINITION

JustSlide is a fully usable, fast, memory-conscious desktop web browser in which both the human user and authorized AI agents are first-class operators of the *same* browser application.

Our central product promise: **Everything the user can reasonably do through JustSlide's own browser features should have a well-defined, permission-controlled programmatic API.**

This does not mean granting arbitrary agents direct access to the operating system, passwords, private tabs, webview internals, or raw browser storage. The API is intentionally bounded and auditable.

The product must provide:

- Real webpage rendering and navigation, not an iframe-based simulation.
- Native multi-tab and multi-window browsing.
- Persistent workspaces with separate visible tab collections.
- Pinned tabs, favorites, bookmarks, history, downloads, search, session recovery, and browser settings.
- Customizable appearance without baking one background color into the brand.
- Native desktop interactions and correct keyboard focus.
- A workspace dock for personal and agent-oriented workspaces.
- Local MCP connectivity usable by compatible clients (including Claude Code, Codex, Antigravity, Cursor, and other MCP clients) with explicit authorization.
- Safe, concurrent background automation that does not interrupt a person actively browsing.
- A path toward WebExtensions and a separate JustSlide plugin SDK, acknowledging actual platform compatibility limits.
- A first-class developer/test workflow and honest compatibility reports.

The main differentiator is *not* a chatbot sidebar. A ChatGPT website bookmark is not a ChatGPT-agent connection. An agent workspace is not merely a folder of website shortcuts. Keep these concepts distinct everywhere.

### 1.1 Platform strategy

- Phase 1: macOS native runtime, with explicit investigation of Intel x86_64 and Apple Silicon arm64.
- Future: Windows and Linux may use their own platform rendering adapters. Do not claim they work before implementation.
- Avoid assuming macOS deployment targets, AppKit APIs, Swift toolchain, or memory limits without checking the actual machine and SDK.
- Use the operating system's WebKit rather than downloading/shipping Chromium.

### 1.2 Quality bar

The user rejected a previous agent's inaccurate browser UI. Thus: **Figma fidelity is a release gate, not optional polish.** If you cannot read the Figma file, do not invent an approximation and call it implemented. Work on independent foundation tasks while resolving Figma access, and record the blocker.

---

## 2. ANTIGRAVITY-SPECIFIC WORKFLOW AND PROJECT CONFIGURATION

Inspect Antigravity's CURRENT official documentation. Use its supported rules, plans, screenshot artifacts, browser subagents, and MCP tooling rather than copying Cursor-only paths.

Relevant official references:

- https://antigravity.google/docs/mcp
- https://antigravity.google/docs/rules/
- https://antigravity.google/docs/agent/
- https://antigravity.google/docs/ide/browser
- https://antigravity.google/docs/screenshots
- https://antigravity.google/docs/walkthrough

### 2.1 Required MCP connections

**Figma Dev Mode MCP is mandatory for UI implementation.** In Antigravity IDE, use the agent panel's `... > MCP Servers` and install/connect **Figma Dev Mode MCP** from the supported MCP Store. Use authenticated user access to the design file. If the connector is already installed, test it before editing the UI. Never invent a Figma API key or silently switch to a CSS-generated approximation.

Before doing UI work, demonstrate successful calls that return:

1. Figma design context for node `1:6` with an actual screenshot.
2. The current hierarchical structure, sizing, and asset references of its key children.
3. Design context or screenshots for the interaction states `30:52` and compact window `35:13`.
4. The selected-state tab, workspace dock, favorites, pinned section, and address-bar details at sufficient resolution.
5. A working permitted asset export/download mechanism for every asset that is actually needed.

If a broad `get_design_context` result is sparse or truncated, request the relevant child node IDs rather than guessing. If the screenshot is absent, call `get_screenshot` explicitly. Keep reference captures for traceability; never use Figma screenshots as the actual UI.

Use the official public ADS docs MCP. The Antigravity workspace configuration format is generally `.agents/mcp_config.json` with `mcpServers`; remote MCP endpoints use `serverUrl` in the official documentation. Merge into the user's existing file; never overwrite unrelated servers. An example for **ADS only**:

```json
{
  "mcpServers": {
    "atlassian-design-system": {
      "serverUrl": "https://mcp.atlassian.com/v1/ads/public/mcp"
    }
  }
}
```

Prefer installing Figma through Antigravity's own store, which handles connector-specific authentication/configuration. Do NOT invent a Figma `serverUrl` or commit bearer tokens to the repository.

Review the Antigravity documentation for the installed version before using any configuration command; if it differs from this example, record the verified form.

### 2.2 Antigravity project rules

Create concise root `AGENTS.md` and modular rules in `.agents/rules/*.md`, not `.cursor/rules/*.mdc`. Antigravity rule files use YAML frontmatter and a valid trigger such as `always_on`, `model_decision`, or `glob`. Verify syntax against current official docs and respect file/context budgets. Suggested rules:

- `.agents/rules/figma-fidelity.md`: source-of-truth, assets, screenshot comparison, no visual invention.
- `.agents/rules/ads-components.md`: design tokens, approved packages, font/license status, no visual framework substitutions.
- `.agents/rules/tauri-security.md`: trusted chrome vs untrusted webpages and capability scopes.
- `.agents/rules/native-webkit.md`: native bridge, threading, focus, profiles, extension investigation.
- `.agents/rules/core-and-mcp.md`: one command model, schema, permissions, event ordering.
- `.agents/rules/testing-evidence.md`: native verification, no fabricated passes.
- `.agents/rules/performance.md`: light footprint, resource budgets, regression checks.

For example, a focused always-on rule should begin:

```md
---
trigger: always_on
description: Non-negotiable project invariants
---
Build JustSlide using WebKit, not Chromium. Treat the current Figma file as
required UI source. No UI feature is complete without a screenshot comparison.
Never grant arbitrary external sites Tauri/native capabilities.
```

Use narrowly scoped rules and reference deeper documents instead of stuffing the entire master prompt into every always-on rule.

### 2.3 Agent orchestration

Use Antigravity's implementation plan/task artifacts and walkthrough artifacts. If supported, delegate independent investigations and tests to subagents, but keep architectural decisions and overlapping files under one coordinator. A Figma extractor, ADS/legal auditor, native-host feasibility engineer, and test engineer can independently investigate; do not let four agents simultaneously rewrite the shell.

A browser subagent may inspect a React development preview using Chrome and capture screenshots, but must not call that a real macOS WKWebView test. Real JustSlide native screenshots and UI tests remain a separate requirement.

After each meaningful milestone, save a checkpoint in `docs/progress.md`: changes, commands, evidence, limitations, and next steps. The project must remain recoverable between model sessions.

---

## 3. ENVIRONMENT AND GIT SAFETY

Before installing or generating files:

- Inspect `pwd`, repository tree, git status, remotes/branches, lockfiles, local assets, current build scripts, and existing Tauri/native code.
- Record actual macOS version, hardware CPU architecture, memory/disk, Xcode/Command Line Tools, Swift, Rust/Cargo, Bun, Node, Git, and available UI automation tools.
- Confirm the worktree is the `JustSlide` repository and not `Slide` or another product.
- Preserve unrelated files and user changes; do not force clean/reset/delete.
- Avoid destructive broad commands, global settings changes, or exposing credentials in logs.
- Isolate development identifiers, browser profiles, SQLite stores, ports, IPC sockets, and build artifacts from Chrome, Safari, Slide, and any previous JustSlide Chromium build.
- If the current repository already contains incomplete Chromium-specific architecture, create a safe migration plan and preserve that work before replacing engine code.
- On a non-macOS environment, perform portable tasks and report that native macOS runtime tests were not run.

Do not require the user to purchase cloud infrastructure, domain services, signing certificates, or additional APIs for the local first milestone. Ask permission before costs, release publishing, external service configuration, or destructive changes.

---

## 4. ARCHITECTURE BASELINE: NO CHROMIUM

Default stack, subject to an early technical proof:

| Concern | Default implementation |
| --- | --- |
| Desktop host | Tauri v2 |
| Trusted browser UI | React + TypeScript |
| UI components | Legally approved Atlassian Design System React components |
| Frontend bundler | Parcel + Atlassian Compiled integration, if compatible with pinned packages |
| JavaScript tooling | Bun; compatible Node tooling where required |
| Application/domain core | Rust |
| Native macOS page host | Swift/AppKit `WKWebView` and a carefully scoped Tauri/Rust bridge |
| Rendering engine | Operating-system WebKit, not Chromium |
| Durable app metadata | SQLite with migrations |
| Browser website data | WebKit website data stores, not DIY cookie tables |
| MCP transport | Rust stdio adapter + authenticated local IPC |
| Browser events | Typed commands, state revisions, subscriptions |
| Mac build verification | Xcode/Swift/Cargo/Tauri native build, platform tests |

Avoid introducing Next.js SSR, Electron, a Chromium runtime, a second browser engine, Redis, PostgreSQL, Docker, an arbitrary Node sidecar, or cloud LLM service into the local first release.

### 4.1 Strict host feasibility gate

Tauri is a product-shell baseline, not a mandate to bury browser limitations under hacks. First build a small native spike proving that its webview model and/or a custom Swift AppKit plugin can deliver the required browser behavior on the target machine:

1. One trusted local React shell and at least two independent external `WKWebView` webpages in a single native window.
2. Correct proportional resize, sidebar collapse, Retina scaling, rounded page container, focus, keyboard shortcuts, accessibility, and page clipping.
3. Navigate a background tab without switching keyboard focus from the human-active page.
4. Layer a native/app-owned menu or permission prompt correctly over the webpage surface.
5. Handle navigation, new windows/popups, downloads, file input, JavaScript dialogs, auth challenges, and failed loads via real callbacks.
6. Capture a screenshot from the requested page, including background capture limitations.
7. Keep untrusted webpages unable to access privileged Tauri commands, filesystem, OS automation, or the app's MCP permission controls.
8. Verify cookies/website data can be persisted and separated by browser profile on the chosen macOS deployment target.
9. Investigate actual `WKWebExtensionController` APIs and deployment target availability.
10. Build the small proof as a real `.app` locally and capture its UI, not a web-only preview.

Tauri's public multiple-webview APIs may have feature gates/limitations. Inspect exact pinned versions and docs; use a native Swift bridge if required. Write `docs/adr/0001-desktop-host.md` with test results.

If Tauri cannot support correct native z-order, input/focus, profile isolation, or extension integration without fragile hacks, propose an **AppKit-owned native host embedding the same React/ADS chrome in a trusted local `WKWebView` and using the same Rust core**. This is still non-Chromium. Preserve app logic/UI architecture; do not silently switch hosts. Request owner approval for an irreversible host change.

### 4.2 Process and trust model

```
Human -> React / ADS (trusted browser chrome)
                       |
                       v
             Typed internal commands
                       |
                       v
       Rust JustSlide Core + permission engine
            |                   |
            v                   v
   macOS WebKit adapter      MCP gateway
   (Swift / AppKit)       (local authenticated)
            |                   |
            v                   v
   Independent WKWebViews    External MCP agents

State events -> Rust event hub -> React subscriptions + authorized MCP observers
```

Rust owns JustSlide's domain commands and persistent app metadata. The native adapter owns actual WKWebView objects and reports real navigation/render state back. The UI never fabricates a successful page load based on a button click.

Swift/WebKit UI-affecting calls run on the main actor/main thread; use typed async boundaries and avoid deadlocks, blocking the UI loop, or sharing raw WebKit handles with Rust workers.

---

## 5. FIGMA SOURCE OF TRUTH - EXACT SCREEN REQUIREMENTS

Current main Figma URL:

`https://www.figma.com/design/nw47hqMLTtmBMksJ4BnJKb/Slide?node-id=1-6&m=dev`

Required nodes:

- `1:6` - Main desktop frame, **1440 x 1024**.
- `1:9` - Sidebar, baseline width **240px**.
- `1:7` - Large rounded webpage window.
- `6:17` - Navigation controls.
- `7:46` - Address bar.
- `7:87` - Website favorites grid.
- `25:4` - Pinned section.
- `20:82` - Scrolling browser tabs area.
- `12:290` - Workspace footer.
- `30:52` - Interaction-state board.
- `35:13` - Compact **1100 x 720** window example with background agent activity.

These IDs reflect the currently inspected file; verify the latest Figma structure rather than trusting hardcoded IDs if the design changed.

Measurements observed from the current source (to verify at build time):

- Desktop frame ~1440 x 1024.
- Outer inset/padding ~12px; sidebar-to-webpage gap ~12px.
- Sidebar ~240px wide.
- Webpage container corner radius ~16px.
- Navigation row ~40px tall; primary hit targets ~32px.
- Address input ~240 x 32 with 8px-ish rounding.
- Favorites grid ~5 columns; tiles ~43 x 34; ~4px gaps; site icons ~18px.
- Pinned section contains a 28px header, ~32px rows with ~4px vertical spacing, count and expand/collapse affordance.
- Tab rows ~32px high, selected state uses a light/white rounded surface and visible close action.
- Tab list scrolls independently; workspace dock stays anchored to the bottom.
- Bottom dock shows a selected Personal workspace and workspace shortcuts for Codex, Claude Code, and ChatGPT, plus downloads/new workspace controls. Workspace target sizes ~32px.
- Original example background #ACFFF7 is a **user-selected theme example**, NOT a fixed global brand color.
- Existing Figma text uses SF Pro; product instruction explicitly supersedes this for the *approved* ADS typography implementation. Resolve approved type scale/font availability without silently claiming parity.

These are reference measurements, not permission to reproduce every Figma class name verbatim. Use real auto/flex layout and ADS sizing where appropriate. Do not implement a 1440px absolute canvas.

### 5.1 Precisely preserve the information architecture

Top to bottom, main left sidebar:

1. Sidebar toggle, back, forward, reload.
2. Address/search bar with actual page URL and load/security state.
3. **Website favorites**: YouTube, ChatGPT, Claude, Gmail, AWS, etc., and Add Favorite tile.
4. Collapsible Pinned section (persistent tabs in the current workspace).
5. Divider, New Tab control, and individually scrollable ordinary tabs.
6. Conditional agent-activity summary when an agent is working.
7. Bottom **WORKSPACES** heading, current workspace name, Personal/Codex/Claude Code/ChatGPT workspace buttons, downloads, and new-workspace control.

**Critical semantic distinction:** top ChatGPT and Claude are website favorites and open sites; bottom ChatGPT and Claude Code are workspaces that select different browser context/tab groups. Do NOT replace the duplicated icons with a generic AI sidebar. The duplicate visuals are intentional because their meanings differ by location.

Use these accessible labels at minimum:

- `Open ChatGPT website`
- `Open Claude website`
- `Switch to Personal workspace`
- `Switch to Claude Code workspace`
- `Switch to ChatGPT workspace`
- `Create new workspace`

An agent workspace icon does NOT prove that an agent is connected or active. Add status only when an actual authenticated agent is working.

### 5.2 Design parity manifest

Before modifying UI code, generate `docs/figma-inventory.md` and a machine-readable `docs/figma-manifest.json` with actual source node IDs, dimensions, visual states, assets, labels, relationships, and timestamp. Include direct source URLs and screenshots. Map every visible element to a specific implementation component and route.

Create `docs/figma-to-ads-mapping.md` with columns:

`Figma Node | Intended Function | Figma Geometry | ADS Component/Primitive | Token/Type Style | Interaction | Asset Path | Implementation File | Verification`

Every visible source asset (favicons, product logos, icons, selected-tab icon, etc.) must be accounted for. Download approved Figma assets through the supported connector and save to stable local paths. Do not leave expiring Figma MCP URLs in runtime source. Respect image/logo licensing; use a legally valid alternative where required, document it, and seek approval rather than drawing a random replacement.

Implement layout as real reusable components, not static screenshot backgrounds or a raw absolute-position recreation of Figma-generated markup.

### 5.3 Hard visual regression contract

**Never mark the UI complete without actual side-by-side screenshots at 1440 x 1024 and 1100 x 720.**

Build a deterministic UI fixture/preview mode independent of real browsing content and secure credentials, with exactly the reference sample tab names, pinned items, favorites, and workspace state. This mode is *development only* and must never be mistaken for real browser state in production.

For each composition:

1. Fetch the current Figma screenshot using Figma MCP.
2. Capture the rendered React preview at the exact viewport/density using Antigravity browser tools or Playwright.
3. Capture the running native JustSlide macOS `.app` at the same window bounds/scale when available.
4. Produce a side-by-side visual diff and, when technically possible, an alpha overlay or pixel/geometry analysis.
5. Compare region bounds, alignment, gaps, typography hierarchy, backgrounds, icon sizes, clipping, borders, radius, focus/selection state, and footer anchoring.
6. Fix the largest mismatches and repeat until layout, content, and assets agree with the source, except approved/recorded ADS font/token differences.
7. Save all baseline, preview, native, and final screenshots under `docs/evidence/visual/` with dates and screen-size labels.
8. Record mismatch count, accepted exceptions, and any unavailable tests in `docs/design-fidelity.md`.

Do not define an arbitrary '95% similarity' metric and call it passing without meaningful region-level checks. Use geometry tolerances and manual/automated visual review. Pixel-perfect differences due to the mandated font/system WebKit should be explicitly documented.

**If a screenshot clearly does not resemble the Figma layout, stop feature expansion and repair it.**

---

## 6. ATLASSIAN DESIGN SYSTEM - AUTHENTIC USE, LEGAL GATE

Primary references:

- https://atlassian.design/
- https://atlassian.design/get-started/develop/atlassians
- https://atlassian.design/get-started/develop/with-ai
- https://atlassian.design/llms.txt
- https://atlassian.design/DESIGN.md
- https://atlassian.design/foundations/tokens/use-tokens-in-code/
- https://atlassian.design/foundations/typography/
- https://atlassian.design/components/
- https://atlassian.design/license/
- https://mcp.atlassian.com/v1/ads/public/mcp

Consult the public ADS MCP/CLI for actual component APIs, theme tokens, typography, accessibility, and version-specific guidance. Do not assume private Atlassian skills or gated tooling are available to an independent app developer.

### 6.1 Licensing must be audited

The overall Atlassian Design System license has a limited scope relating to Atlassian-integrated add-ons. Individual `@atlaskit/*` code packages and assets may have separately applicable open-source licenses. Do not treat the website as a blanket permission to copy every font, icon, asset, or exact visual system into an unrelated commercial browser.

Audit specific versions, transitive dependencies, LICENSE/NOTICE files, fonts, icon sets, and source assets in `docs/licensing-and-assets.md`. Label each `approved`, `needs owner/legal review`, or `blocked`. Do not redistribute unapproved resources. If exact ADS use is not demonstrably permitted for JustSlide, describe the restriction clearly and propose a legally clear implementation using appropriately licensed pieces and original design tokens, without pretending it is full ADS. Await a product-owner decision on blocked shipping assets; continue independent browser-engine work.

### 6.2 Implementation constraints

- Prefer official, permitted React-based ADS components and `@atlaskit/primitives/compiled` as supported.
- Use valid semantic color, spacing, border, radius, elevation, focus, typography, and motion tokens.
- Verify React version and each package's peer dependencies; do not force a newer major with install flags.
- Use compatible Parcel/Compiled/Babel integration where required; verify production CSS extraction and token transforms.
- Configure theme initialization and make sure the stylesheets and typography actually load.
- For small browser-specific controls (tabs, favorites, workspace dock), compose from approved primitives rather than misusing an ADS component with the wrong semantics.
- Do not add Tailwind, shadcn, Material UI, Chakra, or a second competing design system.
- Do not hand-author dozens of unrelated hex colors, shadow recipes, and typographic sizes. Write a small Slide-specific token layer only for genuinely app-specific sizing and user wallpaper/backdrop settings.
- Native OS/browser permission dialogs may need AppKit rendering; document justified exceptions.

### 6.3 Fonts

Current ADS app typography targets **Atlassian Sans** and **Atlassian Mono**. Verify their legitimate availability and usage/redistribution terms. Do not scrape protected fonts, substitute SF Pro/Inter silently, or claim exact ADS typography when a fallback is in use. Keep temporary local development fallbacks explicitly labeled and block final design/production signoff until resolved.

### 6.4 Custom theme contract

Make backgrounds customizable using a separate JustSlide surface/backdrop preference, with preset colors and optional custom color where accessible. Keep text, focus, safety, error, and interaction states based on ADS semantic roles and tested contrast. Support light, dark, and system themes. Browser UI theme must not inject styles into external websites. Make sure appearance persists between restarts without a visible flash of unthemed UI.

---

## 7. BROWSER UI IMPLEMENTATION SPECIFICATION

Construct reusable React components under coherent feature folders. Suggested components:

`SlideWindow`, `SidebarShell`, `NavigationControls`, `AddressSearchBar`, `FavoriteGrid`, `FavoriteTile`, `PinnedSection`, `PinnedTabRow`, `NewTabAction`, `ScrollableTabList`, `BrowserTabRow`, `AgentActivitySummary`, `WorkspaceDock`, `WorkspaceSwitchButton`, `WebContentHostAnchor`, `ToastHost`, `ContextMenuHost`, `PermissionPromptHost`, `NativePageErrorView`, `NewTabPage`, `DownloadsView`, `HistoryView`, `SettingsView`, `ExtensionsView`.

Preserve faithful interactions and keyboard accessibility:

- Navigation control disabled states based on history availability; reload/stop behavior while loading.
- Address bar reflects the real currently selected page. Support user editing and secure URL/search resolution; do not overwrite user-entered text on every minor navigation event.
- Tab selected, hover, pressed, keyboard-focused, loading, muted, suspended, pinned, private, and error visual states where applicable.
- Reorder, select, pin/unpin, duplicate, move-to-workspace, close, reopen; accessible context menus and keyboard alternatives to drag.
- URL/title/favicon updates from real WebKit events, not hardcoded fixture data.
- Pinned section count and expand/collapse persist appropriately.
- Independently scrolling tab list; no sidebar footer displacement at short window heights.
- Workspace dock shows selected workspace and connected/active agent status only when true.
- Agent status card is minimal, opt-in/conditional, and must not cover the page unexpectedly. Pause and permission controls must perform real actions.
- New-tab page is functional and restrained, with search/address input, favorites and optional supplied mascot; no large AI-generated dashboard.
- Correct app zoom, find-in-page, history, downloads, native menus, full-screen, and screen-reader labels as features are implemented.

Use a stable internal layout for native page positioning; shell React and webpage WKWebView should coordinate bounds without React DOM manipulating raw native views. Avoid a foreign WebView overlapping React menus/click targets. Verify actual native hit-testing and z-order for dropdowns and permission prompts.

Do not use macOS web content itself as the React UI's iframe. Real websites should be isolated native WebKit views, with no access to browser chrome's Tauri commands. Do not place the React app over a screenshot of another browser and declare completion.

---

## 8. BROWSER ENGINE ADAPTER - WEBKIT DETAILS

Implement a typed `BrowserEngine` abstraction so core commands do not depend directly on Tauri APIs or Swift implementation details. The macOS `WebKitBrowserAdapter` must handle:

- Creating/destroying `WKWebView` instances with explicit identity and lifecycle.
- Selecting, hiding, loading, navigating, and restoring views with safe focus behavior.
- Tracking provisional navigation, committed navigation, redirects, load progress, URL/title/favicon, errors, and process termination.
- Back/forward lists, reload/stop, history contributions, and open-in-new-tab/window policies.
- Navigation with appropriate URL scheme validation, origin reporting, and TLS behavior.
- Cookies/website data via supported WebKit data stores, with profile isolation verified on the selected SDK.
- Downloads, user-selected file inputs, printing/PDF, clipboard, zoom, native dialogs, popup handling, permission requests, and web inspector/debugging where supported.
- Screenshots via supported native APIs, including reporting when hidden/background capture is unsupported or produces stale content.
- WebKit JS execution for tightly scoped page automation, with cancellation/navigation identity checks, CSP/isolated-world considerations, and explicit permissions.
- Page lifecycle and cache/idle policies, and renderer crash/recovery.
- Native menu items and OS accessibility behavior.

Expose capabilities and platform limitations through a typed feature registry, not guesses. `BrowserEngineCapabilities` should document support for screenshots, extensions, private profiles, downloads, Web Inspector, tab suspension, tab capture, accessibility snapshots, and page automation.

For each unsupported feature, preserve a safe disabled/error state, not a fake working button.

---

## 9. STATE, STORAGE, COMMANDS, AND EVENT SYNCHRONIZATION

Rust JustSlide Core is the source of truth for app-owned window, workspace, tab metadata, favorites, bookmarks, preferences, agent grants, and lifecycle state. WebKit remains authoritative about rendering and real navigation. Keep UI as a projection of domain state and renderer events.

Define stable entities and IDs:

- `Profile`: persistent/private, data-store boundary, permissions, name.
- `Workspace`: label, icon, ordering, profile reference, optional agent owner.
- `BrowserWindow`: currently selected workspace/tab and geometry.
- `Tab`: persistent ID, workspace/profile, URL, display title, favicon reference, pinned/order, active/loading/suspended status, renderer handle, navigation generation.
- `Favorite`: profile-wide or workspace-scoped as documented, ordered website shortcut.
- `Bookmark`: title, URL, folder, tags/metadata when supported.
- `HistoryEntry`: URL, title, timestamp, profile scope, privacy rules.
- `Download`: target, progress, state, source, user authorization.
- `AgentPrincipal`: authenticated identity, local session, grants, resource limits.
- `ActivityRecord`: permitted command, outcome, resource, timestamp, redacted metadata.

Commands share one typed schema with actor identity, request ID, target state revision/precondition, permission requirement, result/error, and emitted event. UI buttons and MCP methods call the same handler. Do not implement separate tab managers for Rust, React, and an MCP sidecar.

Use a versioned protocol, idempotency for safe retry, bounded event queues, stale-navigation checks, snapshot-plus-subscription handshakes, reconnect/resynchronization, and structured failure states.

Store JustSlide metadata in transactional SQLite migrations. Do not copy raw cookies/passwords or WebKit secret stores into application tables. Private sessions should not persist regular browsing history or sensitive grants. Use platform-protected secret storage for local pairing tokens.

Implement recovery for application restart, abrupt termination, missing page data store, failed migration, renderer crash, and duplicate event delivery.

---

## 10. AGENT-NATIVE BROWSER MODEL

Every *supported* native browser operation should be reachable through a versioned programmatic command with an explicit permission policy and tests. Not every OS-private implementation detail should be exposed.

Initial planned command families:

| Domain | Examples |
| --- | --- |
| Capabilities | inspect supported tools, engine features and errors |
| Tabs | list/create/activate/close/reopen/pin/unpin/move/duplicate/mute/suspend |
| Workspaces | list/create/rename/switch/order/delete subject to approval |
| Favorites | list/add/remove/order |
| Bookmarks | add/list/search/move/delete |
| Navigation | navigate/back/forward/reload/stop/find/zoom |
| Page inspection | safe DOM/accessibility snapshot, screenshot, title/URL, selection |
| Page interaction | scoped click/type/scroll/select/file upload under policy |
| Windows | list/create/focus/size when authorized |
| History | search/read/delete with grant checks |
| Downloads | list/start/cancel/inspect permitted files |
| Extensions | list/status/enable/disable/install via approvals when actually supported |
| Settings | read/update permitted preferences and theme |
| Activity | current tasks/permissions/logs/pause/revoke |

Start with a small verified subset. Tool discovery must only announce tools that are actually implemented and working.

### 10.1 Example expected behavior

The person is typing on a personal tab. Claude Code asks JustSlide to create `Research`, open three URLs in that workspace, pin one, and capture the second.

- The tabs appear in the dock/sidebar as changes are committed.
- The selected Personal workspace/tab and keyboard first responder remain unchanged.
- The agent's background tasks are visible in an understated activity state.
- The agent cannot read or change the personal tab without an explicit grant.
- The screenshot returns the requested research tab, not the foreground personal tab.
- No site page can see the privileged command/permission bridge.

This is our defining end-to-end acceptance scenario.

### 10.2 Agent scheduler

Use bounded concurrency and priority scheduling, with human interaction at highest priority. Background agent navigation, DOM inspections, screenshots, and open tabs should have configurable limits. Queue operations when busy, cancel obsolete work, and release resources when idle. Never claim tab discarding is lossless; protect forms, media, downloads, and active interactions. Do not spawn a separate browser engine/profile per agent by default.

---

## 11. MCP INTEGRATION AND CLIENT SUPPORT

Implement `justslide-mcp` as a small Rust stdio MCP executable which forwards to the running JustSlide app over secure local IPC. The browser is the authority; the MCP binary is not an automation-only shadow browser.

- Verify the current MCP protocol and SDK version.
- Use protocol-only stdout and logs on stderr.
- Make instance discovery explicit; never attach to the wrong browser/profile.
- Authenticate clients and identify actors server-side rather than trusting caller-provided `agentId`.
- Use short-lived scoped grants, optional user confirmation, revocation, and redacted activity history.
- Do not open an unauthenticated HTTP automation port on localhost or grant `mcp(*)` blanket permissions by default.
- Export typed tools and useful error codes; validate URLs and input sizes.
- Include timeouts, cancellation, backpressure, and connection restoration.
- Test at least one local MCP client if available and otherwise use a protocol test harness.
- Provide setup instructions for Antigravity, Claude Code, Codex, and Cursor without requiring or asserting each client has connected successfully.
- Remote/cloud-hosted clients need an explicitly designed authenticated bridge; don't assume they can reach a laptop's local process. Do not deploy a relay without owner approval.

A plausible example command for the domain (not a promise of a preexisting API):

```json
{
  "operation": "tabs.create",
  "requestId": "agent-research-001",
  "target": {"workspaceId": "claude-research"},
  "input": {"url": "https://example.com", "activate": false}
}
```

Require a permission matrix for each tool: default state, profile/workspace/origin scope, read/write/sensitive categories, approval path, audit log, and error behavior.

---

## 12. EXTENSIONS AND PLUGINS: HONEST COMPATIBILITY

A non-Chromium WebKit app **cannot be assumed to support every Chrome extension**. This is a known architectural tradeoff. Do not falsely promise one-click installation of all Chrome Web Store packages.

Investigate current Apple `WKWebExtension`, `WKWebExtensionController`, `WKWebExtensionContext`, `WKWebExtensionTab`/window delegates, APIs, minimum deployment targets, entitlements, App Store/distribution rules, and the compatibility status of Manifest V3 extension features.

Build an extension feasibility experiment using a lawful, simple fixture. Track support for:

- Content scripts and isolation.
- Runtime messaging and storage.
- Service workers/background logic.
- Toolbar actions/popups/options pages.
- Tabs/windows events and workspace mapping.
- Permissions and origin grants.
- Native messaging if relevant and permitted.
- Extension installation, update, enable/disable, and removal.

Maintain `docs/extension-matrix.md` with each feature as supported/partially supported/blocked/unknown, the actual native test and macOS/SDK version. Only show active extension-management actions when implemented.

Distinguish:

1. WebExtensions running in WebKit.
2. The custom JustSlide plugin SDK for app commands/UI contributions.
3. The MCP API for external AI clients.

They have different security and distribution boundaries. A WebExtension permitted in a profile may access more than one workspace in that profile; never imply a workspace visual grouping is an extension sandbox.

If the owner insists on near-universal Chrome extension compatibility, report that WebKit may not meet that requirement and request an explicit architectural tradeoff decision. Do not secretly reintroduce Chromium.

---

## 13. SECURITY, PERMISSIONS, PRIVACY, AND TRUST

Trust boundaries are: external webpages, website renderer/page scripts, trusted React UI, Tauri/Rust core, native Swift bridge, local MCP process, externally connected agents, installed extensions/plugins, OS credentials, and persistent storage.

Non-negotiable protections:

- External sites are untrusted native WebKit views, never the privileged Tauri chrome.
- Scope all Tauri capabilities to trusted UI origin/webview only. Do not use a broad window/webview wildcard when it includes page content.
- Check actor permission at Rust command handlers, not only in hidden/disabled UI controls.
- Never expose native file access, shell command execution, passwords, arbitrary OS controls, raw cookies, or unguarded JavaScript evaluation to pages/agents.
- Verify popup/frame navigation cannot acquire privileged local origins, scripts, or capabilities.
- Honor TLS verification, WebKit security guarantees, and sandbox/entitlement restrictions.
- Treat webpage text, DOM, downloads, screenshots, and MCP tool output as untrusted data, not instructions that grant more authority.
- Require a user-controlled approval for cross-profile access, active personal tab changes, extension installation, destructive operations, sensitive page submissions, and downloads/uploads where appropriate.
- Use bounded local IPC, tokens stored in platform-protected storage, least privilege and immediate future-action revocation.
- Redact logs, and avoid retaining page content, screenshots, full URLs with secrets, passwords, and private browsing history without an explicit reason/consent.
- Distinguish automatic safe page reading from consequential operations such as submitting payments, sending messages, or changing accounts.
- Make an agent's pause/status visible and accessible. Do not let the agent approve its own escalation.

Build adversarial tests where an untrusted page tries to invoke Tauri commands, escalate MCP access, or persuade an agent to reveal another profile's tabs.

---

## 14. PERFORMANCE FIRST - LIGHTWEIGHT NATIVE WEBKIT

JustSlide must strive for low installed size, low idle CPU, smooth tab switching, and responsible memory use on Intel Macs as well as Apple Silicon. Do not assume Tauri is automatically efficient simply because it uses system WebKit.

Performance strategy:

- Reuse the OS WebKit runtime rather than bundling Chromium.
- Avoid duplicate webview instances for preview/screenshot generation.
- Create webpages lazily when background tabs only need metadata, and report `not loaded` truthfully.
- Cap active agent tasks, screenshots, and background navigations.
- Use WebKit lifecycle/resource APIs supported by the target macOS, with validated safe unload/restore policies.
- Prioritize active typing/media/user interactions; never forcibly discard active forms/downloads.
- Cache website icons/tab metadata with explicit limits.
- Lazy-load settings, downloads, history, extension manager, and other secondary React views.
- Avoid large frontend dependency graphs, redundant contexts/stores, excessive render subscriptions, giant SVG icon packs, per-tab timers, and constant screenshot polling.
- Measure app plus WebKit networking/content/GPU process memory rather than just the Tauri parent process.
- Minimize JavaScript bridge chatter by batching state revisions and avoiding huge DOM snapshots.
- Use separate development/debug and production packaging configurations.
- Ship only required assets and legal font files; never package unnecessary test/build artifacts.

Establish a repeatable baseline before aggressive optimizations. Measure cold/warm startup, time to usable window, tab switch latency, idle/active CPU, aggregate process memory, installed `.app` size, and performance with 1/5/20/50 tabs and 0/1/3 agents. Record CPU, memory, OS, power source, and versions. No fabricated performance claims or arbitrary numeric budgets without measurements.

---

## 15. QUALITY ENGINEERING AND FIDELITY GATES

### 15.1 Frontend verification

- React component tests for tabs, favorites, controls, dock, states and accessibility.
- Deterministic design fixture with reference content that cannot leak into production.
- Screenshot tests at Figma dimensions, light/dark/selected/hover/focus, narrow/short windows.
- Verify each required Figma SVG/logo asset appears in the correct slot with correct dimensions, not a guessed icon.
- Compile and inspect production CSS/font/theme assets, not only dev server.
- Test keyboard-only navigation, focus trap, reduced motion, screen-reader labels, tooltips, and contrast.

### 15.2 Native verification

- Compile `.app` for the actual macOS development architecture.
- Run native WebKit tabs, navigation, downloads, dialogs, session persistence, and focus tests.
- Screenshot the actual native window; do not reuse browser preview screenshots.
- Test multi-display/scaling where available; no unsafe first responder changes.
- Verify Tauri capabilities cannot be invoked from untrusted webpages.
- Restore and reconcile tabs after app restart and renderer process crash.
- Verify two independent profiles do not accidentally share website data.
- Validate the exact background-tab screenshot behavior rather than relying on assumptions.

### 15.3 Core/MCP tests

- Rust unit/integration tests for commands, migrations, permissions, event ordering, idempotency, concurrency, cancellation, and recovery.
- MCP protocol harness checks for schema, denied operations, token expiry, revocation, and malformed input.
- Human-versus-agent focus scenario with a stable local test webpage that records continuous typed input and focus events.
- Negative tests for page prompt injection, agent access to other workspaces/profiles, and stale navigation targets.

### 15.4 Evidence/reporting

Each milestone reports exact commands, exit codes, passing/failing/not-run tests, macOS/arch, source revision, UI screenshots, native screenshots, limitations, and next steps. Never describe a preview as a working browser or a theoretical capability as implemented. Antigravity walkthroughs should attach screenshots where supported.

---

## 16. DOCUMENTATION - REQUIRED, NOT EMPTY BOILERPLATE

Create and maintain these files as appropriate for the codebase:

- `README.md`: what actually works, prerequisites, run/build/test steps, limitations.
- `AGENTS.md`: short essential invariants and references.
- `DESIGN.md`: official JustSlide design contract (not falsely named as an Atlassian-authored file).
- `docs/prd.md`: users, differentiators, MVP, product non-goals, roadmap, acceptance.
- `docs/architecture.md`: processes, trust boundaries, code ownership, UI-to-engine integration.
- `docs/platform-adapter.md`: native WebKit bridge, views, profiles, threading, limitations.
- `docs/figma-inventory.md`: actual selected design nodes and assets.
- `docs/figma-manifest.json`: machine-readable geometry, tokens, assets, reference states.
- `docs/figma-to-ads-mapping.md`: every visible UI slot and its implementation.
- `docs/design-fidelity.md`: visual diffs, accepted mismatches, theme/font decisions.
- `docs/design-system.md`: ADS components, token mapping, typography license, icons.
- `docs/licensing-and-assets.md`: per-package/asset right-to-use evidence.
- `docs/security-and-privacy.md`: threat model, grants, policies, logs/retention.
- `docs/data-and-protocol.md`: entities, command/event schemas, revisions, migrations.
- `docs/mcp-and-plugins.md`: supported tools, pairing, local/remote separation, extensions.
- `docs/extension-matrix.md`: WebKit extension results and support levels.
- `docs/performance-engineering.md`: methodology, baseline, budgets, optimizations.
- `docs/testing-and-evidence.md`: UI/native/MCP test plan and outputs.
- `docs/development-and-release.md`: Xcode, signing, Intel/arm64, CI, packaging, update strategy.
- `docs/adr/`: host, browser engine, WebKit bridge, ADS/licensing, profile isolation, MCP transport, threading.
- `docs/sources.md`: official references with dates and pinned versions.
- `docs/plan.md`: ordered milestones/dependencies.
- `docs/progress.md`: current implemented status and reproducible resume commands.

Use succinct, precise documents with links to source and tests. Do not output a hundred pages of filler while postponing the working application.

---

## 17. PROJECT STRUCTURE

Inspect existing structure before adding anything. A sensible greenfield layout is:

```text
JustSlide/
  AGENTS.md
  DESIGN.md
  README.md
  package.json
  bun.lock
  Cargo.toml
  rust-toolchain.toml
  .agents/
    mcp_config.json
    rules/
      figma-fidelity.md
      ads-components.md
      tauri-security.md
      native-webkit.md
      core-and-mcp.md
      performance.md
      testing-evidence.md
  apps/
    desktop/
      src-tauri/
        src/
        capabilities/
        tauri.conf.json
    browser-chrome/
      src/
        app/
        shell/
        features/
        bridge/
        styles/
  native/
    macos/
      WebKitBrowserAdapter/
  crates/
    justslide-core/
    justslide-protocol/
    justslide-mcp/
  packages/
    contracts/
    ui/
  assets/
    mascot/
    icons/
  tests/
    fixtures/
    integration/
    e2e/
    visual/
  docs/
    adr/
    evidence/
      visual/
      native/
      test-results/
  scripts/
```

Create only modules that are part of the current milestone. Resist empty architectural cosplay. Preserve the ability to build a functional small vertical slice with clear dependency boundaries.

---

## 18. ENGINEERING PRACTICES AND SOURCE CONTROL

- Use Bun/TypeScript in frontend and tooling where appropriate; Rust for core and MCP; Swift/AppKit for native WebKit integration.
- Type every bridge command and event; generate frontend contracts from a canonical source where feasible.
- Keep production and test fixtures separately.
- Use meaningful errors, typed Result-style paths, structured redacted diagnostics, and clean cancellation.
- Avoid blocking native main thread with SQLite, shell, or network tasks.
- Do not add unknown Rust unsafe code without a documented necessity and review.
- Keep active app state revisions immutable/transactional to prevent UI desync.
- No hardcoded secrets, broad filesystem permissions, package-install workarounds, or silent telemetry.
- Keep live user website data out of repo/tests/logs.
- Write tests alongside each feature and keep the app launchable between milestones.
- Respect current native/ADS API signatures; inspect docs and source before implementing unfamiliar calls.
- Pin and record verified package/tool versions and lockfiles.
- Format/lint/typecheck/test/build after meaningful change sets.
- Make small meaningful commits/checkpoints when authorized; do not push or publish automatically.

---

## 19. BUILD SEQUENCE AND MUST-PASS MILESTONES

### M0 - Discovery and safe start

Confirm correct repo, current work, machine, toolchain, source docs, Figma access, ADS licensing constraints, Antigravity project configuration. Produce initial `docs/plan.md` and architectural ADR candidates. Do not write frontend visual code until Figma MCP returns current design context and screenshot.

**Pass:** readable source inventory, functioning Figma MCP read, actual screenshot, environment report, separate JustSlide identity.

### M1 - Native WebKit feasibility spike

Implement two isolated external WKWebViews with a trusted local React shell. Prove navigation, focus preservation, menus/overlays, secure capabilities, profile data stores, screenshots, and native build. Document WebKit/Tauri limitations and choose host path.

**Pass:** real macOS app and evidence; no fake page preview.

### M2 - ADS and visual shell

Configure verified, legally usable ADS React packages and bundled styles/fonts. Recreate current Figma layout using reusable components and actual assets. Compare 1440x1024 and 1100x720 screenshots, fix visible mismatches. Support customizable backdrop, light/dark, active/hover/focus states, independent tab scrolling and fixed workspace dock.

**Pass:** real screenshot evidence and explicit ADS license/font status. Do not confuse a nice sample mock with browser functionality.

### M3 - Real basic browser

Wire address/search, navigation, tab operations, favorites, pinned section, workspace CRUD/select, persistent state, page state, error/loading, and restoration. Remove fake data from production startup.

**Pass:** navigate real sites and restart without losing approved state.

### M4 - MCP and live state

Add a thin Rust MCP bridge, local pairing, scoped grants, typed commands/events, real screenshots, activity state, pause/revoke. Prove background-agent creation/pin without focus theft.

**Pass:** local MCP test harness creates three real tabs without interrupting typing.

### M5 - Native browser polish

Add working history/bookmarks/downloads/settings/find/zoom/shortcuts, private session rules, native menus, error recovery, accessibility refinements.

**Pass:** core day-to-day browser tasks work in the native app, not preview only.

### M6 - WebExtensions feasibility

Validate the chosen OS/SDK's extension APIs and a legal MV3 fixture; build an honest compatibility matrix and basic user-authorized lifecycle controls where supported.

**Pass:** documented tested support and explicit unsupported features, not blanket Chrome Web Store claims.

### M7 - Performance, distribution, release prep

Baseline/fix slow paths, resource manager and agent scheduler, architecture-specific builds, repeatable test/CI gates, signing/notarization plan and update strategy.

**Pass:** reproducible native builds, quantified performance and known release blockers. No unsolicited public deployment.

Do not stop after M0 or M2. Complete the highest possible verifiable vertical slice within the current working session. If a license, environment, or connector restriction blocks one milestone, explain precisely and continue independent nonblocked work.

---

## 20. FINAL ACCEPTANCE TESTS

The following are mandatory before labeling the first JustSlide usable milestone complete:

**Design:**

1. Match the current main Figma frame in real layout at 1440x1024.
2. Match compact 1100x720 composition with pinned collapsed and agent activity state.
3. Show exact semantic difference between site favorites and workspace dock.
4. Correct active tab, address URL, and displayed webpage correspondence.
5. Stable bottom workspace dock while tab list scrolls.
6. Visually verified icons/brand assets; no substituted generic icons.
7. Approved font and ADS token/component status documented.
8. Theme/backdrop customization persists and maintains contrast.

**Browser:**

9. Launch native `.app`, navigate HTTPS, use back/forward/reload.
10. Create/pin/move/close/reopen tabs and select workspaces.
11. Persist favorites, tabs and workspace order across restart.
12. Show real load/error/navigation state; no stale fake address.
13. Handle new windows/popups and downloads with correct permission flow.
14. Correct renderer recovery, tab restoration, active form/audio protection.

**Agents/security:**

15. Agent creates background tabs while human types; focus and active tab unchanged.
16. Two agents receive only authorized workspace/profile scope.
17. Page screenshot returns the actual requested tab or a truthful unsupported error.
18. Revoked MCP token cannot subsequently mutate/read browser state.
19. A hostile website cannot call native Tauri/Rust methods or approve grants.
20. A page's injected instructions cannot grant access to passwords, personal tabs or files.

**Platform/quality:**

21. Build and test on the actual Mac architecture; document other architectures as unverified until tested.
22. Frontend preview and native WebKit screenshots clearly distinguished.
23. Accessibility and keyboard navigation pass relevant tests.
24. Aggregate memory/CPU/startup measurements recorded with methodology.
25. Production bundle contains no development-only fixtures, secret credentials, external font downloads or unconstrained debugging server.

---

## 21. EXPECTED HANDOFF FROM ANTIGRAVITY

At the end of every completed milestone return a concise but evidence-based walkthrough:

1. What changed (file paths and key decisions).
2. Which planned vs implemented features work now.
3. The exact commands to run the app and tests on the verified host.
4. Screenshots: Figma reference, React preview, native macOS app, comparison.
5. Test/build results with pass/fail/not-run, architecture, environment, logs.
6. ADS package usage, rights/typography status, and visual exceptions.
7. Confirmed browser engine, WebKit native adapter and profile isolation status.
8. Local MCP commands verified and permission/agent behavior.
9. Extension compatibility outcomes/unknowns.
10. Performance baseline and major regressions/optimizations.
11. Outstanding blockers, risks, and next three actions.
12. Updated `docs/progress.md` with reproducible resume instructions.

Do not hide failed tests or say 'everything works' based on a frontend screenshot. Do not promise work in a future unseen run; complete real implementation tasks and leave concrete evidence during this session.

---

## 22. BEGIN NOW - EXACT FIRST ACTIONS

Execute this list in order, without asking the owner to repeat already provided product/design information:

1. Inspect `JustSlide` repo, branch, git status and host toolchain.
2. Determine whether previous Chromium work is present, and preserve it before any migration.
3. Verify Antigravity **Figma Dev Mode MCP** by reading node `1:6` and capturing its screenshot. Read nodes `30:52`, `35:13`, and key children.
4. Verify public ADS MCP/CLI and inspect licensing and typography source availability.
5. Generate the design inventory and implementation matrix from source evidence.
6. Inspect Tauri v2/native WebKit support and document the feasibility spike plan.
7. Create short project rules, initial ADRs, and docs/plan/progress.
8. Build/execute the native WebKit spike and run safety/focus tests.
9. Implement the shell, capture a side-by-side Figma comparison, and fix mismatches.
10. Continue into real tabs/workspaces and an initial local MCP smoke test, maintaining test evidence.

**Remember: JustSlide is NOT a Chromium browser. It is the same agent-native product, built with system WebKit and a native Rust/Swift/Tauri architecture, and it must faithfully implement the actual Figma design with legally approved Atlassian Design System components.**