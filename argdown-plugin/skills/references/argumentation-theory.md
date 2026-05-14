# Argumentation theory primer

Three frames the v0.2 skills invoke. ≤100 words per theorist.

## Pollock — rebutting vs undercutting

A **rebutting defeater** attacks a conclusion directly ("not-P"). An **undercutting defeater** attacks the inferential link between premises and conclusion without asserting the opposite conclusion. In Argdown, label the mode explicitly:

```argdown
[P]: Rain fell last night.
  -> [C]: The ground is wet.
     - [Rebutter]: The ground has a drain cover. // attacks C directly
     - [Undercutter]: The observation was indoors. // attacks P->C link
```

Pollock argued undercutting is often more economical when a conclusion has heavy independent support.

> Source: [Stanford Encyclopedia of Philosophy — Defeasible Reasoning](https://plato.stanford.edu/entries/reasoning-defeasible/) <!-- source-verified: HTTP 200 via curl on 2026-05-13 -->

## Govier — ARG conditions

Trudy Govier's ARG triad evaluates premises on three axes: **Acceptability** (the audience can grant it without begging the question), **Relevance** (it bears on the conclusion), and **Grounds** (sub-arguments or evidence support it). The `find-unsupported-premises` skill flags nodes that fail any axis:

```argdown
[P]: Studies show X. // Grounds: cite source; Acceptability: contested — flag
  -> [C]: Policy Y is justified.
```

A premise that satisfies Grounds but not Acceptability still fails the triad.

> Source: [Wikipedia — Trudy Govier](https://en.wikipedia.org/wiki/Trudy_Govier) <!-- source-verified: HTTP 200 via curl on 2026-05-13 -->

## Toulmin — claim/data/warrant/backing

Toulmin's model assigns each statement a structural role: **claim** (conclusion asserted), **data** (facts cited in support), **warrant** (inference rule linking data to claim), **backing** (authority or evidence supporting the warrant). The `extract-argument` skill labels statements accordingly when converting prose to Argdown:

```argdown
[Claim]: The bridge is safe.
  <+ [Data]: Load tests passed.
     <+ [Warrant]: Tests follow EN 1337 standard.
        <+ [Backing]: EN 1337 is the applicable EU norm.
```

> Source: [Wikipedia — Stephen Toulmin](https://en.wikipedia.org/wiki/Stephen_Toulmin) <!-- source-verified: HTTP 200 via curl on 2026-05-13; https://plato.stanford.edu/entries/argumentation-models/ returned 404 -->
