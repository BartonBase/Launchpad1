# Meteora DBC fixtures (read-only dumps)

- `dbc_mainnet.so`: the DBC program `dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN` (`solana program dump -u mainnet-beta`, 2026-09-25).
- `token_metadata_mainnet.so`: Metaplex Token Metadata `metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`, which DBC CPIs into to create token metadata.
- `devnet_samples.json`: real devnet DBC accounts fetched by RPC (base64):
  - `5L1MfYm4…` PoolConfig: SOL quote, SPL, 6 decimals, fixed 1B supply with pre == post, Immutable. The tests use it as a template and patch `leftover_receiver` and the threshold.
  - `DGtaRQ9E…` VirtualPool (migrated).
  - `9beobQVq…` VirtualPool (not migrated).
  - `7J3ko5BF…` a transfer-hook pool (different discriminator).

Used by `vault/dbc_graduation.rs`. No keys or secrets are stored here.
