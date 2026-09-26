# Switchboard On-Demand devnet fixtures (read-only dumps)

- `sb_devnet.so`: the devnet program `Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2`, dumped with `solana program dump -u devnet`.
- `accounts.json`: devnet accounts fetched by RPC (`getMultipleAccounts`, base64): the queue `EYiAmGSdsQTuCw413V5BzaruWuCCSDgTPtBGvLkXHbe7`, the state PDA `["STATE"]`, the wSOL mint, the queue's oracles, and their `["OracleStats", oracle]` and `["OracleRandomnessStats", oracle]` PDAs. `fetched_slot` and `unix_time` are the snapshot's clock; the harness warps LiteSVM to them so the oracle heartbeats look fresh.

Used by `vault/real_switchboard.rs` (`Env::new_svm_real_sb`). To refresh them, re-dump the program and re-fetch the same keys. No keys or secrets are stored here.
