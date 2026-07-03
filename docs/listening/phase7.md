# Listening Checkpoint 1 (CP1) — Phase 7 prosody

**Status: AWAITING HUMAN SIGN-OFF.** Fill this in after listening, then tag.

- Date:
- Rater:
- Battery: `cargo xtask listening-battery` → artifacts/listening/phase7/index.html
  (10 utterances × 3 renders: voksa prosodic / voksa flat / eSpeak-NG jbo oracle)

## Scores

MOS scale 1–5 (1 = unusable, 3 = acceptable, 5 = natural). ABX: which of
prosodic/flat sounds better. Paste the markdown the index.html button builds:

| slug | MOS intelligibility | MOS naturalness | ABX | notes |
|---|---|---|---|---|
| coi-munje | 1 | 1 | flat | Sounds like soi-oon-shae |
| le-prenu | 4 | 3 | flat | `pr` and `kl` pronouciation strong and long |
| mi-zgana | 3 | 3 | flat | Same as above. seems like `CC` (Consonant-Consonant) bits are a bit stronger and longer than neccesary |
| la-djan | 2 | 2 | flat | CC stong + long. |
| li-pi | 2 | 1 | prosodic | The eSpeak oracle pronounces 4 differently than both voksa |
| nelci-buffer | 1 | 1 | prosodic | same CC stong + long |
| djan-dotside | 4 | 3 | flat | Same CC strong + long |
| xu-rise | 3 | 3 | flat | Same CC strong + long. This ketter than eSpeak. espeak pronounces `xu` like `ku` |
| declarative | 3 | 3 | flat | Now that I think of it, the start of 5/6 letter words is hard + long in beginning |
| djosefin | 3 | 3 | flat | DJO is hard + long |

NOTES FROM THE USER (dhilipsiva): Please do not take these comments and scores at face vaule. I have never heard someone else speak lojban other than me. So I might have done something teribbly wrong. Mostly I listened to eSpeak first (assuming it is accurate) and then juded the MOS int + MOS nat. So there is a chance that I might be very wrong.


## Specific checks

- [ ] Declination audible on `declarative` (sentence "settles" toward the end)?
- [ ] Stressed syllables prominent (DJOsefin, klama) without sounding shouted?
- [ ] `xu-rise` clearly a question vs `declarative`?
- [ ] `nelci-buffer` buffer vowels unobtrusive?
- [ ] Pauses (la-djan, djan-dotside) natural, not stuttery?
- [ ] Anything worse than the flat baseline? (should be "no" everywhere)

## Verdict

- [ ] PASS — tag: `git tag -a phase7-complete -m "Phase 7: prosody + CP1 sign-off" && git push origin phase7-complete`
- [ ] ITERATE — list items below; Phase 7 continues.

Iteration items / deferred notes:
-
