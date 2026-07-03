//! CLL §3.9 syllabification.
//!
//! Deterministic algorithm choosing CLL's "normal" readings (documented in
//! docs/phonology.md §3): nuclei are vowels/diphthongs munched left-to-right
//! (CLL §3.5), each inter-nucleus consonant run gives its maximal legal-onset
//! suffix to the following syllable (coinciding with CLL's C.C / .CC / C.CC
//! rules for all standard words), and sonorants become syllabic nuclei ONLY
//! in vowel-less regions (CLL §3.4 leaves syllabicity to the speaker; this
//! choice is stress-invariant per §3.9).

use crate::alloc::string::String;
use crate::alloc::vec::Vec;
use crate::clusters::{is_legal_onset, is_sonorant};
use crate::letters::{Letter, WordError, consonant_to_char, parse_word, vowel_to_char};
use crate::phonemes::{Consonant, Vowel, is_valid_diphthong};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nucleus {
    Vowel(Vowel),
    Diphthong(Vowel, Vowel),
    /// A syllabic sonorant (l m n r) — never stressed, never counted (CLL §3.9).
    Syllabic(Consonant),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Syllable {
    pub onset: Vec<Consonant>,
    /// True when the syllable is introduced by an apostrophe (= [h]).
    pub aspirated: bool,
    pub nucleus: Nucleus,
    pub coda: Vec<Consonant>,
}

/// Syllabify one word (lowercase letters + apostrophe + comma).
pub fn syllabify(word: &str) -> Result<Vec<Syllable>, WordError> {
    let letters = parse_word(word)?;
    let mut syllables = Vec::new();
    for segment in letters.split(|l| *l == Letter::Comma) {
        syllabify_segment(segment, &mut syllables)?;
    }
    Ok(syllables)
}

fn syllabify_segment(letters: &[Letter], out: &mut Vec<Syllable>) -> Result<(), WordError> {
    // Pass 1: nuclei (left-to-right maximal diphthong munch, CLL §3.5) and the
    // consonant runs around them. runs[k] precedes nucleus k; the final run is
    // the trailing coda.
    struct Proto {
        aspirated: bool,
        nucleus: Nucleus,
    }
    let mut runs: Vec<Vec<Consonant>> = Vec::new();
    runs.push(Vec::new());
    let mut protos: Vec<Proto> = Vec::new();
    let mut aspirate_next = false;
    let mut i = 0;
    while i < letters.len() {
        match letters[i] {
            Letter::C(c) => {
                runs.last_mut().expect("runs never empty").push(c);
                i += 1;
            }
            Letter::V(v1) => {
                let nucleus = match letters.get(i + 1) {
                    Some(Letter::V(v2)) if is_valid_diphthong(v1, *v2) => {
                        i += 2;
                        Nucleus::Diphthong(v1, *v2)
                    }
                    _ => {
                        i += 1;
                        Nucleus::Vowel(v1)
                    }
                };
                protos.push(Proto {
                    aspirated: aspirate_next,
                    nucleus,
                });
                aspirate_next = false;
                runs.push(Vec::new());
            }
            Letter::Apostrophe => {
                aspirate_next = true;
                i += 1;
            }
            Letter::Comma => unreachable!("segments are comma-split"),
        }
    }

    if protos.is_empty() {
        // Vowel-less segment (e.g. the `r` in kat,r,in. or the word rl.):
        // syllabic sonorants are the only possible nuclei.
        return resolve_residue(&runs[0], out);
    }

    // Pass 2: distribute the consonant runs.
    let n = protos.len();
    let mut built: Vec<Syllable> = protos
        .into_iter()
        .map(|p| Syllable {
            onset: Vec::new(),
            aspirated: p.aspirated,
            nucleus: p.nucleus,
            coda: Vec::new(),
        })
        .collect();
    // Leading run: maximal legal onset; any residue (only possible in cmevla)
    // becomes syllabic-sonorant syllables prepended before the first nucleus.
    let (residue, onset) = split_onset_max(&runs[0]);
    built[0].onset = onset;
    let mut prefix = Vec::new();
    if !residue.is_empty() {
        resolve_residue(&residue, &mut prefix)?;
    }
    // Inter-nucleus runs: maximal legal onset to the right, rest to the coda.
    for k in 1..n {
        let (coda, onset) = split_onset_max(&runs[k]);
        built[k - 1].coda = coda;
        built[k].onset = onset;
    }
    // Trailing run: coda of the final syllable (never broken up — CLL permits
    // arbitrary pairwise-permissible name codas; stress-invariant either way).
    built[n - 1].coda = runs[n].clone();

    out.extend(prefix);
    out.extend(built);
    Ok(())
}

/// Split a consonant run into (remainder, onset) where the onset is the
/// maximal legal suffix (CLL §3.9's C.C / .CC / C.CC rules all fall out).
fn split_onset_max(run: &[Consonant]) -> (Vec<Consonant>, Vec<Consonant>) {
    for start in 0..=run.len() {
        if is_legal_onset(&run[start..]) {
            return (run[..start].to_vec(), run[start..].to_vec());
        }
    }
    unreachable!("the empty suffix is always a legal onset")
}

/// Resolve a consonant region with no adjacent vowel nucleus: each sonorant
/// becomes a syllabic nucleus; stranded obstruents attach forward as onset
/// where the chain is legal, otherwise to the previous syllabic's coda.
fn resolve_residue(run: &[Consonant], out: &mut Vec<Syllable>) -> Result<(), WordError> {
    if run.is_empty() {
        return Ok(());
    }
    let mut made: Vec<Syllable> = Vec::new();
    let mut i = 0;
    while i < run.len() {
        let Some(j) = run[i..].iter().position(|c| is_sonorant(*c)).map(|p| p + i) else {
            // No sonorant left: leftovers join the previous syllabic's coda.
            match made.last_mut() {
                Some(last) => {
                    last.coda.extend_from_slice(&run[i..]);
                    break;
                }
                None => return Err(WordError::NoNucleus),
            }
        };
        // Maximal onset chain into the sonorant nucleus.
        let mut k = j;
        while k > i && is_legal_onset(&run[k - 1..=j]) {
            k -= 1;
        }
        if k > i {
            match made.last_mut() {
                Some(last) => last.coda.extend_from_slice(&run[i..k]),
                None => return Err(WordError::NoNucleus),
            }
        }
        made.push(Syllable {
            onset: run[k..j].to_vec(),
            aspirated: false,
            nucleus: Nucleus::Syllabic(run[j]),
            coda: Vec::new(),
        });
        i = j + 1;
    }
    out.extend(made);
    Ok(())
}

/// Render syllables back to text (apostrophes preserved, commas dropped) —
/// the round-trip counterpart of [`syllabify`].
pub fn to_text(syllables: &[Syllable]) -> String {
    let mut out = String::new();
    for syl in syllables {
        if syl.aspirated {
            out.push('\'');
        }
        for c in &syl.onset {
            out.push(consonant_to_char(*c));
        }
        match syl.nucleus {
            Nucleus::Vowel(v) => out.push(vowel_to_char(v)),
            Nucleus::Diphthong(a, b) => {
                out.push(vowel_to_char(a));
                out.push(vowel_to_char(b));
            }
            Nucleus::Syllabic(c) => out.push(consonant_to_char(c)),
        }
        for c in &syl.coda {
            out.push(consonant_to_char(*c));
        }
    }
    out
}
