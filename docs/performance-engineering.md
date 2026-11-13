# Performance engineering

Baseline (2026-11-13, Linux x86_64 sandbox, 2 vCPU, release build, TEST-DOUBLE engine — measures JustSlide's own command path, not WebKit):
`cargo run --release -p justslide-core --features test-engine --example bench` → `docs/evidence/test-results/core-bench-2026-11-13.txt`

| Tabs | tabs.create (incl. SQLite txn) | state.snapshot | RSS |
|---|---|---|---|
| 1 | 0.36 ms | 0.04 ms | 7.7 MB |
| 5 | 0.69 ms | 0.07 ms | 7.8 MB |
| 20 | 0.80 ms | 0.16 ms | 8.2 MB |
| 50 | 1.61 ms | 0.34 ms | 9.0 MB |

Open + migrate + seed: 4 ms. `justslide-mcp` release binary: 4.5 MB. Chrome production bundle: 1.2 MB (46 files incl. all ADS theme chunks).

Known cost: persistence rewrites the small app tables on each mutation (linear in tabs) — switch to incremental writes before 200+ tabs. Not yet measured (needs the Mac): cold/warm start, time to usable window, tab switch latency, idle CPU, aggregate app + WebKit WebContent/Networking/GPU process memory with 1/5/20/50 tabs and 0/1/3 agents, installed .app size. Method: `xcrun xctrace` (Activity Monitor template) + `footprint` per process, on AC power, recorded with macOS/arch/Xcode.
