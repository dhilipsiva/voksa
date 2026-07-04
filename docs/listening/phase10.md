# Listening Checkpoint 2 (CP2) — Phase 10 attitudinals

**Status: AWAITING HUMAN SIGN-OFF.** Fill this in after listening, then tag.

- Date:
- Rater:
- Battery: `cargo xtask attitudinal-battery` → artifacts/listening/phase10/index.html
  (7 items × 3 renders: **voksa (affect)** = prosody + attitudinal overlay /
  **neutral (base)** = same words minus the UI cmavo / **eSpeak-NG jbo oracle**)

> The attitudinal overlay is **INVENTED / non-normative** — the CLL specifies no
> acoustic realization for UI cmavo (a `.ui` is defined by its *meaning*, joy,
> not a pitch or voice quality). The deviation vectors, intensity multipliers,
> and word-scope rule are voksa's own (docs/phonology.md §10). Judge whether the
> intended emotion is *recognizable* and whether the voice still sounds natural.

## Scores

MOS naturalness 1–5 (1 = unusable, 3 = acceptable, 5 = natural). In **heard**,
write the emotion you actually perceive when you play *voksa (affect)* against
its *neutral* baseline — ideally blind. Paste the markdown the index.html button
builds:

| slug | heard emotion | MOS naturalness | notes |
|---|---|---|---|
| joy-ui |  |  |  |
| complaint-oi |  |  |  |
| fear-ii |  |  |  |
| sadness-uu |  |  |  |
| patience-oo |  |  |  |
| desire-au |  |  |  |
| anger-oonai |  |  |  |

## Specific checks

- [ ] `joy-ui` reads brighter / higher / livelier than its neutral base?
- [ ] `sadness-uu` reads lower, slower, and breathier?
- [ ] `fear-ii` has an audible flutter (vibrato) and sounds tense/high?
- [ ] `complaint-oi` / `anger-oonai` have audible creak (diplophonia)?
- [ ] `patience-oo` reads calm / flat (monotone) without sounding dead?
- [ ] Each affect render is distinguishable from its neutral baseline?
- [ ] Any affect that sounds WORSE (less natural) than its neutral base?

## Verdict

- [ ] PASS — tag: `git tag -a phase10-complete -m "Phase 10: attitudinals + CP2 sign-off" && git push origin phase10-complete`
- [ ] ITERATE — list items below; Phase 10 continues (deviation-vector tuning is
      a rules-only lever, safe to iterate without re-architecting).

Iteration items / deferred notes:
-
