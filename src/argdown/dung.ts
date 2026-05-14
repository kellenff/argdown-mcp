/**
 * Dung-style grounded extension over an Argdown document's argument graph.
 *
 * Algorithm: Caminada three-valued labelling fixpoint (Caminada 2006,
 * "On the issue of reinstatement in argumentation"). Each argument is
 * labelled IN, OUT, or UNDEC. The grounded extension is the unique minimal
 * complete labelling — equivalently, the least fixed point of the
 * characteristic function F(S) = { a : every attacker of a is in S^+ }.
 *
 * Iteration rules:
 *   - An UNDEC argument becomes IN when every attacker is OUT
 *     (vacuously: an unattacked argument has no attackers, all-OUT is trivially true).
 *   - An UNDEC argument becomes OUT when at least one attacker is IN.
 *   - Repeat until a pass makes no changes. Remaining UNDEC argments are UNDEC.
 *
 * Self-attack convention: a self-attacker has itself as one of its
 * attackers. Since it starts UNDEC, neither rule fires for it directly:
 *   - "all attackers OUT" fails (its self-attacker is UNDEC, not OUT)
 *   - "some attacker IN" fails (it is UNDEC, not IN)
 * So an isolated self-attacker remains UNDEC.
 *
 * A self-attacker also attacked from outside by an IN argument becomes OUT
 * via the second rule (the external IN attacker triggers it).
 *
 * Attack relation is `[attacker, attacked]`. Self-loops are allowed.
 *
 * Verified empirically against Argdown core 2.0.1 (see .relation-shape.md):
 *   - argument-level attacks via `<X>\n  - <Y>` produce
 *     IRelation { relationType: "attack", from: <argument Y>, to: <argument X> }
 *   - statement-level attacks stay as equivalence-class relations and
 *     are NOT auto-lifted into argument attacks
 */

import type { IArgdownResponse } from "@argdown/core";
import { ArgdownTypes, RelationType, type IArgument } from "@argdown/core";

export type DungLabel = "in" | "out" | "undec";

export type DungExtension = {
  in: string[];
  out: string[];
  undec: string[];
};

export type DungResult = {
  extension: DungExtension;
  argumentCount: number;
  attackCount: number;
};

/**
 * Extract argument-to-argument attack edges from an Argdown response.
 *
 * Filters strictly on:
 *   - relationType === ATTACK (excludes support, undercut, contrary, contradictory)
 *   - from.type === ARGUMENT
 *   - to.type === ARGUMENT
 *
 * Returns `[attacker.title, attacked.title]` pairs. Self-loops permitted.
 */
export function extractArgumentAttacks(
  resp: IArgdownResponse,
): Array<[string, string]> {
  const out: Array<[string, string]> = [];
  for (const r of resp.relations ?? []) {
    if (r.relationType !== RelationType.ATTACK) continue;
    if (r.from?.type !== ArgdownTypes.ARGUMENT) continue;
    if (r.to?.type !== ArgdownTypes.ARGUMENT) continue;
    const fromTitle = (r.from as IArgument).title;
    const toTitle = (r.to as IArgument).title;
    if (typeof fromTitle !== "string" || typeof toTitle !== "string") continue;
    out.push([fromTitle, toTitle]);
  }
  return out;
}

/**
 * Compute the grounded extension via Caminada three-valued labelling.
 *
 * @param argumentIds  Distinct argument titles. Order is preserved in output.
 * @param attacks      `[attacker, attacked]` pairs. Attackers/attacked outside
 *                     `argumentIds` are ignored (defensive).
 */
export function computeGrounded(
  argumentIds: string[],
  attacks: Array<[string, string]>,
): DungExtension {
  const known = new Set(argumentIds);
  const label = new Map<string, DungLabel>();
  for (const id of argumentIds) label.set(id, "undec");

  const attackersOf = new Map<string, string[]>();
  for (const id of argumentIds) attackersOf.set(id, []);
  for (const [from, to] of attacks) {
    if (!known.has(from) || !known.has(to)) continue;
    attackersOf.get(to)!.push(from);
  }

  let changed = true;
  while (changed) {
    changed = false;
    for (const id of argumentIds) {
      if (label.get(id) !== "undec") continue;
      const attackers = attackersOf.get(id)!;
      const allOut = attackers.every((a) => label.get(a) === "out");
      if (allOut) {
        label.set(id, "in");
        changed = true;
        continue;
      }
      const someIn = attackers.some((a) => label.get(a) === "in");
      if (someIn) {
        label.set(id, "out");
        changed = true;
      }
    }
  }

  const ext: DungExtension = { in: [], out: [], undec: [] };
  for (const id of argumentIds) {
    ext[label.get(id)!].push(id);
  }
  return ext;
}

/**
 * End-to-end: take an Argdown response, return grounded extension + counts.
 */
export function dungGrounded(resp: IArgdownResponse): DungResult {
  const argumentIds = Object.keys(resp.arguments ?? {});
  const attacks = extractArgumentAttacks(resp);
  const extension = computeGrounded(argumentIds, attacks);
  return {
    extension,
    argumentCount: argumentIds.length,
    attackCount: attacks.length,
  };
}
