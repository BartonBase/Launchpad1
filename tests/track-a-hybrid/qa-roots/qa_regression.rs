#![allow(non_snake_case)]
//! Engineer-owned test root so `cargo test` / `./scripts/test.sh` run QA's regression suite
//! (tests/regression, QA-owned, not edited here). Mirrors tests/regression/qa_regression_mod.rs;
//! same relative #[path]s because this file sits at the same depth. Add new QA files here too.
#[path = "../vault/harness.rs"]
mod harness;
#[path = "../../regression/common.rs"]
mod common;
#[path = "../../regression/qa_M01_A01_escrow_drain.rs"]
mod qa_m01_a01;
#[path = "../../regression/qa_M01_A02_token_swap.rs"]
mod qa_m01_a02;
#[path = "../../regression/qa_M02_A03_cherrypick.rs"]
mod qa_m02_a03;
#[path = "../../regression/qa_M14_escrow_sequence.rs"]
mod qa_m14_seq;
#[path = "../../regression/qa_M05_M06_M36_sol_fee.rs"]
mod qa_sol_fee;
#[path = "../../regression/qa_M10_M22_graduation.rs"]
mod qa_m10_m22_grad;
