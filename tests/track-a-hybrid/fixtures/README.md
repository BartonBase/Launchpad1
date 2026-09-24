# Test fixtures (LiteSVM only)

| File | What | Provenance |
|---|---|---|
| `mpl_core.so` | Metaplex Core program binary, loaded at `CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d` in LiteSVM | `solana program dump -u devnet CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d` on 2026-09-24 3:05 PM MT. sha256 `5680d2371204033f55cdcf0c08d393ed737d99cfe35d83f65befe80f7537484a` |

The Switchboard stand-in is **not** a fixture: it's the TEST-ONLY `mock-switchboard` crate in this folder, built by
`Anchor.toml [scripts] test` into `target/deploy/mock_switchboard.so` and loaded at the Switchboard devnet program id.
