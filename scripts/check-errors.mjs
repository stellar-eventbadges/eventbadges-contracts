#!/usr/bin/env node
//
// Keeps ERRORS.md in sync with the `#[contracterror] pub enum Error` block in
// `src/types.rs`. No dependencies beyond Node's built-ins.
//
// Usage:
//   node scripts/check-errors.mjs
//
// Exits non-zero when:
//   - a variant exists in the code but has no row in ERRORS.md,
//   - a row exists in ERRORS.md but the variant is not in the code,
//   - a variant has a different code in the two files,
//   - a variant is documented twice, or
//   - either file (or the enum inside `src/types.rs`) cannot be found or parsed.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const CODE_SOURCE = path.join('src', 'types.rs');
export const DOC_SOURCE = 'ERRORS.md';

/**
 * Parses `Variant = <code>,` entries out of the `#[contracterror]` enum in a
 * Rust source string. Throws with a clear message when the enum cannot be
 * found or when a variant has no explicit code.
 */
export function parseCodeErrors(source) {
  const lines = source.split(/\r?\n/);

  const attributeIndex = lines.findIndex((line) => /#\[\s*contracterror\b/.test(line));
  if (attributeIndex === -1) {
    throw new Error(`no #[contracterror] attribute found in ${CODE_SOURCE}`);
  }

  const headerIndex = lines.findIndex(
    (line, index) => index > attributeIndex && /pub\s+enum\s+Error\b/.test(line),
  );
  if (headerIndex === -1) {
    throw new Error(`no \`pub enum Error\` found after the #[contracterror] attribute in ${CODE_SOURCE}`);
  }

  const variants = [];
  for (let index = headerIndex + 1; index < lines.length; index += 1) {
    const lineNumber = index + 1;
    const withoutComment = lines[index].replace(/\/\/.*$/, '');
    if (/^\s*}/.test(withoutComment)) break;

    const withCode = /^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(\d+)\s*,?\s*$/.exec(withoutComment);
    if (withCode) {
      variants.push({ variant: withCode[1], code: Number(withCode[2]), line: lineNumber });
      continue;
    }

    const bareVariant = /^\s*([A-Za-z_][A-Za-z0-9_]*)\s*,?\s*$/.exec(withoutComment);
    if (bareVariant) {
      throw new Error(
        `${CODE_SOURCE}:${lineNumber}: variant \`${bareVariant[1]}\` has no explicit code; ` +
          'write it as `Name = <number>,` so the docs can be checked against it',
      );
    }
  }

  if (variants.length === 0) {
    throw new Error(`found \`pub enum Error\` in ${CODE_SOURCE} but no \`Variant = <code>,\` entries`);
  }

  return variants;
}

/**
 * Parses error-table rows of the form `| <code> | \`Variant\` | ... |` out of a
 * markdown string. Rows whose first cell is not a plain number (for example the
 * `1–9` range rows in the categories table) are ignored.
 */
export function parseDocErrors(markdown) {
  const rows = [];
  const lines = markdown.split(/\r?\n/);

  lines.forEach((line, index) => {
    const match = /^\s*\|\s*(\d+)\s*\|\s*`?([A-Za-z_][A-Za-z0-9_]*)`?\s*\|/.exec(line);
    if (match) {
      rows.push({ code: Number(match[1]), variant: match[2], line: index + 1 });
    }
  });

  return rows;
}

/**
 * Compares a Rust source string against an ERRORS.md string and returns every
 * drift between them.
 */
export function checkSources({ rustSource, docsSource }) {
  const codeErrors = parseCodeErrors(rustSource);
  const docErrors = parseDocErrors(docsSource);

  const codeByVariant = new Map();
  for (const entry of codeErrors) {
    if (!codeByVariant.has(entry.variant)) codeByVariant.set(entry.variant, entry);
  }

  const docByVariant = new Map();
  const duplicates = [];
  for (const entry of docErrors) {
    if (docByVariant.has(entry.variant)) {
      duplicates.push(entry.variant);
      continue;
    }
    docByVariant.set(entry.variant, entry);
  }

  const missingInDoc = codeErrors.filter((entry) => !docByVariant.has(entry.variant));
  const missingInCode = docErrors.filter((entry) => !codeByVariant.has(entry.variant));
  const mismatched = [];
  for (const [variant, codeEntry] of codeByVariant) {
    const docEntry = docByVariant.get(variant);
    if (docEntry && docEntry.code !== codeEntry.code) {
      mismatched.push({
        variant,
        code: codeEntry.code,
        codeLine: codeEntry.line,
        docCode: docEntry.code,
        docLine: docEntry.line,
      });
    }
  }

  return {
    codeErrors,
    docErrors,
    missingInDoc,
    missingInCode,
    mismatched,
    duplicates,
  };
}

/** Reads the two files from a repository root and compares them. */
export function checkRepo(rootDir) {
  let rustSource;
  let docsSource;
  try {
    rustSource = readFileSync(path.join(rootDir, CODE_SOURCE), 'utf8');
  } catch {
    throw new Error(`${CODE_SOURCE} not found under ${rootDir}; nothing to check against`);
  }
  try {
    docsSource = readFileSync(path.join(rootDir, DOC_SOURCE), 'utf8');
  } catch {
    throw new Error(`${DOC_SOURCE} not found under ${rootDir}; run this from the repository root`);
  }
  return checkSources({ rustSource, docsSource });
}

/** True when a `checkSources`/`checkRepo` result contains no drift. */
export function isInSync(result) {
  return (
    result.missingInDoc.length === 0 &&
    result.missingInCode.length === 0 &&
    result.mismatched.length === 0 &&
    result.duplicates.length === 0
  );
}

/** Formats a `checkSources` result as a diff-style report. */
export function formatReport(result, { codeSource = CODE_SOURCE, docSource = DOC_SOURCE } = {}) {
  if (isInSync(result)) {
    return `${docSource} is in sync with ${codeSource} (${result.codeErrors.length} variants checked)`;
  }

  const lines = [`${docSource} is out of sync with ${codeSource}`, ''];

  if (result.missingInDoc.length > 0) {
    lines.push(`  Variants in code, missing from ${docSource}:`);
    for (const entry of result.missingInDoc) {
      lines.push(`    + ${entry.variant} = ${entry.code}  (${codeSource}:${entry.line})`);
    }
    lines.push('');
  }

  if (result.missingInCode.length > 0) {
    lines.push(`  Variants in ${docSource}, missing from the code:`);
    for (const entry of result.missingInCode) {
      lines.push(`    - ${entry.variant} = ${entry.code}  (${docSource}:${entry.line})`);
    }
    lines.push('');
  }

  if (result.mismatched.length > 0) {
    lines.push('  Codes that differ between the two files:');
    for (const entry of result.mismatched) {
      lines.push(
        `    ~ ${entry.variant}: code says ${entry.code}, ${docSource} says ${entry.docCode}` +
          `  (${codeSource}:${entry.codeLine}, ${docSource}:${entry.docLine})`,
      );
    }
    lines.push('');
  }

  if (result.duplicates.length > 0) {
    lines.push(`  Variants documented more than once in ${docSource}:`);
    for (const variant of result.duplicates) {
      lines.push(`    ! ${variant}`);
    }
    lines.push('');
  }

  lines.push('Fix the file that is wrong, then run this check again.');
  return lines.join('\n');
}

function main() {
  const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  try {
    const result = checkRepo(rootDir);
    const report = formatReport(result);
    if (isInSync(result)) {
      console.log(report);
      return 0;
    }
    console.error(report);
    return 1;
  } catch (error) {
    console.error(`check-errors: ${error.message}`);
    return 1;
  }
}

const invokedDirectly =
  process.argv[1] !== undefined &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);

if (invokedDirectly) {
  process.exitCode = main();
}
