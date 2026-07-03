//! Consonant cluster legality tables (CLL §3.6, §3.7).

use crate::phonemes::Consonant;

use Consonant::{B, C, D, F, G, J, K, L, M, N, P, R, S, T, V, X, Z};

/// The 48 valid initial consonant pairs (CLL §3.7, verified verbatim).
pub const INITIAL_PAIRS: [(Consonant, Consonant); 48] = [
    (B, L),
    (B, R),
    (C, F),
    (C, K),
    (C, L),
    (C, M),
    (C, N),
    (C, P),
    (C, R),
    (C, T),
    (D, J),
    (D, R),
    (D, Z),
    (F, L),
    (F, R),
    (G, L),
    (G, R),
    (J, B),
    (J, D),
    (J, G),
    (J, M),
    (J, V),
    (K, L),
    (K, R),
    (M, L),
    (M, R),
    (P, L),
    (P, R),
    (S, F),
    (S, K),
    (S, L),
    (S, M),
    (S, N),
    (S, P),
    (S, R),
    (S, T),
    (T, C),
    (T, R),
    (T, S),
    (V, L),
    (V, R),
    (X, L),
    (X, R),
    (Z, B),
    (Z, D),
    (Z, G),
    (Z, M),
    (Z, V),
];

pub fn is_initial_pair(a: Consonant, b: Consonant) -> bool {
    INITIAL_PAIRS.contains(&(a, b))
}

fn is_voiced(c: Consonant) -> bool {
    matches!(c, B | D | G | J | V | Z)
}

fn is_unvoiced(c: Consonant) -> bool {
    matches!(c, C | F | K | P | S | T | X)
}

fn is_cjsz(c: Consonant) -> bool {
    matches!(c, C | J | S | Z)
}

/// The sonorants that may serve as syllabic nuclei (CLL §3.4).
pub fn is_sonorant(c: Consonant) -> bool {
    matches!(c, L | M | N | R)
}

/// Medial pair permissibility (CLL §3.6): not doubled; no voiced+unvoiced mix
/// (l m n r exempt — they are neither); not both from {c j s z}; and the five
/// specifically forbidden pairs cx kx xc xk mz.
pub fn is_permissible_pair(a: Consonant, b: Consonant) -> bool {
    if a == b {
        return false;
    }
    if (is_voiced(a) && is_unvoiced(b)) || (is_unvoiced(a) && is_voiced(b)) {
        return false;
    }
    if is_cjsz(a) && is_cjsz(b) {
        return false;
    }
    !matches!((a, b), (C, X) | (K, X) | (X, C) | (X, K) | (M, Z))
}

/// The four forbidden triples (CLL §3.7): ndj ndz ntc nts.
pub fn is_forbidden_triple(a: Consonant, b: Consonant, c: Consonant) -> bool {
    matches!((a, b, c), (N, D, J) | (N, D, Z) | (N, T, C) | (N, T, S))
}

/// A consonant run is a legal syllable onset iff every adjacent pair is one of
/// the 48 initial pairs (CLL §4.7: "spraile" is acceptable, "ktraile" is not).
/// Empty and single-consonant onsets are always legal.
pub fn is_legal_onset(run: &[Consonant]) -> bool {
    run.windows(2).all(|w| is_initial_pair(w[0], w[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_48_initial_pairs() {
        assert_eq!(INITIAL_PAIRS.len(), 48);
        assert!(is_initial_pair(S, T));
        assert!(is_initial_pair(T, C));
        assert!(is_initial_pair(Z, V));
        assert!(!is_initial_pair(R, G)); // sairgoi splits sair.goi
        assert!(!is_initial_pair(K, T));
    }

    #[test]
    fn medial_rules_match_cll_3_6() {
        assert!(!is_permissible_pair(B, B)); // doubled
        assert!(!is_permissible_pair(S, D)); // unvoiced + voiced
        assert!(!is_permissible_pair(S, C)); // both from cjsz
        assert!(!is_permissible_pair(C, X)); // specifically forbidden
        assert!(!is_permissible_pair(M, Z)); // specifically forbidden
        assert!(is_permissible_pair(N, R)); // sonorants pair freely
        assert!(is_permissible_pair(L, S));
        assert!(is_permissible_pair(R, M)); // armstrong's coda pair
        assert!(is_permissible_pair(F, T));
    }

    #[test]
    fn forbidden_triples_ndj_ndz_ntc_nts() {
        assert!(is_forbidden_triple(N, D, J));
        assert!(is_forbidden_triple(N, D, Z));
        assert!(is_forbidden_triple(N, T, C));
        assert!(is_forbidden_triple(N, T, S));
        assert!(!is_forbidden_triple(R, D, J)); // lerldjamo's fix is fine
    }

    #[test]
    fn onset_legality_is_pairwise_initial() {
        assert!(is_legal_onset(&[]));
        assert!(is_legal_onset(&[K]));
        assert!(is_legal_onset(&[S, T]));
        assert!(is_legal_onset(&[S, P, R])); // spraile
        assert!(!is_legal_onset(&[K, T, R])); // ktraile
        assert!(!is_legal_onset(&[T, R, K])); // trkaile (rk not initial)
    }
}
