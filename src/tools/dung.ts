/**
 * `dung_extensions` MCP tool — compute the grounded extension under Dung's
 * abstract argumentation framework via Caminada three-valued labelling.
 *
 * Input shape mirrors `parse` / `export_json` exactly (flat `{ kind, source,
 * path }`). The same handler-side validation of the kind/source/path
 * invariant applies.
 *
 * Semantics:
 *   - Arguments-only attack relation (statement-level attacks ignored;
 *     Argdown does NOT auto-lift them — verified empirically in
 *     `.relation-shape.md`).
 *   - Grounded extension only (no preferred / stable / complete /
 *     semi-stable / ideal semantics).
 *   - Full IN/OUT/UNDEC partition exposed (Approach C per plan-2026-05-14-0011).
 */

import { runInline } from "../argdown/inline.js";
import { runFile } from "../argdown/file.js";
import { dungGrounded } from "../argdown/dung.js";
import { shapeDungResponse, type McpToolResult } from "../shape.js";
import { type Input } from "./shared.js";

export async function dispatchDung(input: Input): Promise<McpToolResult> {
  if (input.kind === "inline" && !input.source) {
    return {
      isError: true,
      content: [
        {
          type: "text",
          text: "Invalid input: kind='inline' requires a non-empty `source` field.",
        },
      ],
    };
  }
  if (input.kind === "file" && !input.path) {
    return {
      isError: true,
      content: [
        {
          type: "text",
          text: "Invalid input: kind='file' requires a non-empty `path` field.",
        },
      ],
    };
  }

  try {
    const response =
      input.kind === "inline"
        ? runInline(input.source!, { withJson: false })
        : await runFile(input.path!, { withJson: false });
    const dung = dungGrounded(response);
    return shapeDungResponse(response, dung);
  } catch (err) {
    return {
      isError: true,
      content: [
        {
          type: "text",
          text: `Internal error: ${err instanceof Error ? err.message : String(err)}`,
        },
      ],
    };
  }
}
