#![allow(non_snake_case)]
//! Test-binary root for QA regression tests (compiled only by tests/regression/run-against-wip.sh).
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
#[path = "../../regression/qa_M22_lazy_mint.rs"]
mod qa_m22_lazy;
#[path = "../../regression/qa_M10_M22_graduation.rs"]
mod qa_m10_m22_grad;
