---
trigger: model_decision
description: Read when changing persistence, snapshots, event fan-out, tab lifecycle or bundle size.
---
- Measure before optimising: `cargo run --release -p justslide-core --features test-engine --example bench`; native app/WebKit process memory per `docs/performance-engineering.md`.
- Restored/background tabs stay `notLoaded` until needed; never create duplicate WebKit views for previews or captures.
- Coalesce chrome refreshes (one snapshot per event burst); cap agent captures in flight (2) and tab creation (30/min).
- Keep the chrome bundle lean; lazy-load secondary views.
