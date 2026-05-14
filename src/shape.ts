import type {
  IArgdownResponse,
  IEquivalenceClass,
  IStatement,
} from "@argdown/core";

export type ShapeMode = "parse" | "export_json";

export type McpToolResult = {
  isError?: boolean;
  content: Array<{ type: "text"; text: string }>;
  structuredContent?: Record<string, unknown>;
};

/**
 * Project an Argdown response into an MCP tool result.
 *
 * Single code path for both `parse` and `export_json` modes; the only
 * mode-specific behaviour is whether the `resp.json` payload is appended.
 *
 * Statement counting uses a Set of source-location tuples rather than
 * `Object.keys(resp.statements).length`, because Argdown's equivalence
 * classes merge by `[Title]` across files: two `@import`-merged statements
 * with the same title collapse to one key but retain distinct member
 * locations. The set-of-locations count is the honest answer to
 * "how much input was parsed".
 *
 * `@argdown/core` does not record per-statement source filenames (the
 * IncludePlugin in `@argdown/node` performs textual substitution before
 * parsing), so the location key is the chevrotain offset tuple itself —
 * `startLine:startColumn:endLine:endColumn:startOffset` — which uniquely
 * identifies each member's span in the combined input buffer.
 *
 * Verified against:
 *   - @argdown/core/dist/model/model.d.ts (HasLocation, IStatement,
 *     IEquivalenceClass, ISection)
 *   - @argdown/core/dist/index.d.ts (IArgdownResponse base)
 *   - @argdown/core/dist/plugins/ParserPlugin.d.ts (lexerErrors,
 *     parserErrors, tokens augmentation)
 *   - @argdown/core/dist/plugins/ModelPlugin.d.ts (arguments, statements,
 *     sections: ISection[] — an array, not a record)
 *   - @argdown/core/dist/plugins/JSONExportPlugin.d.ts (json?: string)
 *   - @chevrotain/types/api.d.ts (ILexingError { line, column, message },
 *     IRecognitionException { token: IToken, message })
 */
export function shapeResponse(
  resp: IArgdownResponse,
  mode: ShapeMode,
): McpToolResult {
  const statements = resp.statements ?? {};
  const args = resp.arguments ?? {};
  const sections = resp.sections ?? [];
  const tokens = resp.tokens ?? [];

  const lexerErrors = resp.lexerErrors ?? [];
  const parserErrors = resp.parserErrors ?? [];
  const exceptions: Error[] = resp.exceptions ?? [];

  // Count statements by distinct source-location across equivalence-class members.
  const locationKeys = new Set<string>();
  for (const key of Object.keys(statements)) {
    const ec = statements[key]!;
    for (const m of ec.members ?? []) {
      locationKeys.add(locationKey(m));
    }
  }

  const statementCount = locationKeys.size;
  const argumentCount = Object.keys(args).length;
  const sectionCount = sections.length;

  const lines: string[] = [];
  lines.push(
    `Parsed ${statementCount} statements (by source-location), ${argumentCount} arguments, ${sectionCount} sections.`,
  );

  const hasDiagnostics =
    lexerErrors.length > 0 || parserErrors.length > 0 || exceptions.length > 0;

  if (hasDiagnostics) {
    lines.push("Errors:");
    for (const e of lexerErrors) {
      lines.push(`  lex L${e.line}:${e.column}: ${e.message}`);
    }
    for (const e of parserErrors) {
      const t = e.token;
      lines.push(`  parse L${t?.startLine}:${t?.startColumn}: ${e.message}`);
    }
    for (const e of exceptions) {
      lines.push(`  exception: ${e.message}`);
    }
  }

  // "Not valid Argdown" hint: @argdown/core 2.x is extremely permissive — it
  // parses arbitrary prose into anonymous "untitled statements" with auto-
  // generated titles of the form "Untitled N" (capital U, single space, integer
  // counter). A statementCount===0 check therefore almost never fires for
  // free-form text. The honest heuristic: declare the input Argdown only if it
  // contains at least one *titled* statement, an argument, a section, or a
  // relation. Synthetic-title pattern verified by local probe against
  // @argdown/core 2.0.1: `runInline("Just a sentence.", ...).statements`
  // yields key/title "Untitled 1"; valid `[Hello]: world` yields title "Hello".
  const syntheticTitle = /^Untitled \d+$/;
  const hasTitledStatement = Object.values(statements).some((ec) => {
    const title = (ec as IEquivalenceClass).title;
    return typeof title === "string" && !syntheticTitle.test(title);
  });
  const hasArgument = argumentCount > 0;
  const hasSection = sectionCount > 0;
  const hasRelation = (resp.relations ?? []).length > 0;
  const looksLikeArgdown =
    hasTitledStatement || hasArgument || hasSection || hasRelation;

  const inputWasEmpty = tokens.length === 0;
  const notValidArgdown =
    !inputWasEmpty && !looksLikeArgdown && !hasDiagnostics;

  if (notValidArgdown) {
    lines.push("(not valid Argdown)");
  }

  if (mode === "export_json") {
    const json = resp.json;
    if (json === undefined || json === "") {
      lines.push("JSON: (empty)");
    } else {
      lines.push("JSON:");
      lines.push("```json");
      lines.push(json);
      lines.push("```");
    }
  }

  const isError = hasDiagnostics || notValidArgdown;

  const result: McpToolResult = {
    content: [{ type: "text", text: lines.join("\n") }],
  };
  if (isError) {
    result.isError = true;
  }
  return result;
}

function locationKey(s: IStatement): string {
  return `${s.startLine ?? ""}:${s.startColumn ?? ""}:${s.endLine ?? ""}:${s.endColumn ?? ""}:${s.startOffset ?? ""}`;
}
