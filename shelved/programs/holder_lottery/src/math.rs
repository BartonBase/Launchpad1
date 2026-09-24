//! Pure, side-effect-free lottery math (unit-tested on the host).

/// Number of tickets for a balance: `floor(balance / threshold)`.
///
/// Returns `None` when `threshold == 0` (never divide by zero). Remainders are
/// discarded, which makes wallet-splitting useless:
/// `floor(a/t) + floor(b/t) <= floor((a+b)/t)` for all `a, b` and `t > 0`.
pub fn tickets_for_balance(balance: u64, threshold: u64) -> Option<u64> {
    balance.checked_div(threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_threshold_is_rejected() {
        assert_eq!(tickets_for_balance(1_000, 0), None);
    }

    #[test]
    fn floors_and_discards_remainder() {
        assert_eq!(tickets_for_balance(0, 100), Some(0));
        assert_eq!(tickets_for_balance(99, 100), Some(0));
        assert_eq!(tickets_for_balance(100, 100), Some(1));
        assert_eq!(tickets_for_balance(199, 100), Some(1));
        assert_eq!(tickets_for_balance(u64::MAX, 1), Some(u64::MAX));
    }

    /// Splitting a balance across wallets never increases the ticket count.
    #[test]
    fn splitting_across_wallets_gives_no_advantage() {
        let t = 1_000u64;
        let mut seed = 0x9E37_79B9_7F4A_7C15u64; // deterministic xorshift
        for _ in 0..10_000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let total = seed % 50_000_000;
            let parts = 1 + (seed % 7);
            // split `total` into `parts` uneven pieces
            let mut remaining = total;
            let mut split_tickets = 0u64;
            for i in 0..parts {
                let piece = if i + 1 == parts { remaining } else { remaining / 2 };
                remaining -= piece;
                split_tickets += tickets_for_balance(piece, t).unwrap();
            }
            assert!(split_tickets <= tickets_for_balance(total, t).unwrap());
        }
    }
}
