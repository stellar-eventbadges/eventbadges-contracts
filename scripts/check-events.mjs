#!/usr/bin/env node
//
// Keeps `docs/events.md` in sync with the `#[contractevent]` structs in
// `src/types.rs`. No dependencies beyond Node's built-ins.
//
// Usage:
//   node scripts/check-events.mjs
//
// Exits non-zero when:
//   - a `#[contractevent]` struct has no `## \`Name\`` section in the docs,
//   - a documented section has no struct in the code,
//   - a struct field marked `#[topic]` is not listed in the section's Topics
//     row, or an unmarked field is not listed in its Data row (and the same
//     checks in reverse),
//   - a documented section has no Topics or Data row at all,
//   - a struct or a section is documented twice, or
//   - either file (or the structs inside `src/types.rs`) cannot be found or
//     parsed.
//
// The docs must list every field as `` `field` (`Type`) `` — the shape every
// row in `docs/events.md` already uses. A field written any other way reads as
// missing, which is the point: the row has to say the type, and the type has
// to match `src/types.rs`.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const CODE_SOURCE = path.join('src', 'types.rs');
export const DOC_SOURCE = path.join('docs', 'events.md');

/** Field references of the form `` `name` (`Type`) `` inside a table cell. */
function parseFieldRefs(cell) {
  const fields = [];
  const pattern = /`([a-z_][a-z0-9_]*)`\s*\(`([^`]+)`\)/g;
  let match = pattern.exec(cell);
  while (match !== null) {
    fields.push(match[1]);
    match = pattern.exec(cell);
  }
  return fields;
}

/**
 * Parses `#[contractevent]` structs out of a Rust source string, splitting
 * their fields into the topics the SDK derives from `#[topic]` and the
 * remaining data-map fields.
 */
export function parseCodeEvents(source) {
  const lines = source.split(/\r?\n/);
  const events = [];
  let index = 0;

  while (index < lines.length) {
    if (!/^\s*#\[\s*contractevent\b/.test(lines[index])) {
      index += 1;
      continue;
    }

    const attributeLine = index + 1;
    let header = index + 1;
    while (header < lines.length && !/^\s*pub\s+struct\b/.test(lines[header])) {
      header += 1;
    }
    if (header >= lines.length) {
      throw new Error(
        `${CODE_SOURCE}:${attributeLine}: #[contractevent] is not followed by a \`pub struct\``,
      );
    }

    const headerMatch = /^\s*pub\s+struct\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{/.exec(lines[header]);
    if (!headerMatch) {
      throw new Error(
        `${CODE_SOURCE}:${header + 1}: expected \`pub struct Name {\` on one line after #[contractevent]`,
      );
    }

    const name = headerMatch[1];
    const topics = [];
    const data = [];
    let pendingTopic = false;
    let closed = false;

    for (let i = header + 1; i < lines.length; i += 1) {
      const line = lines[i];
      if (/^\s*\}/.test(line)) {
        index = i + 1;
        closed = true;
        break;
      }
      if (/^\s*#\[\s*topic\s*\]/.test(line)) {
        pendingTopic = true;
        continue;
      }
      const field = /^\s*pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:/.exec(line);
      if (field) {
        (pendingTopic ? topics : data).push({ name: field[1], line: i + 1 });
        pendingTopic = false;
      }
    }

    if (!closed) {
      throw new Error(`${CODE_SOURCE}:${header + 1}: struct ${name} is never closed`);
    }

    events.push({ name, topics, data, line: header + 1 });
  }

  if (events.length === 0) {
    throw new Error(`found no #[contractevent] structs in ${CODE_SOURCE}`);
  }

  return events;
}

/**
 * Parses `## \`EventName\`` sections out of the docs, taking the field names
 * from each section's Topics and Data rows.
 */
export function parseDocEvents(markdown) {
  const lines = markdown.split(/\r?\n/);
  const events = [];
  let current = null;

  lines.forEach((line, index) => {
    const heading = /^##\s+`([A-Za-z_][A-Za-z0-9_]*)`\s*$/.exec(line);
    if (heading) {
      current = {
        name: heading[1],
        topics: [],
        data: [],
        hasTopicsRow: false,
        hasDataRow: false,
        line: index + 1,
      };
      events.push(current);
      return;
    }

    if (current === null) return;

    const row = /^\|\s*(Topics|Data)\s*\|(.*)\|\s*$/.exec(line);
    if (row === null) return;

    const [, label, cell] = row;
    if (label === 'Topics') current.hasTopicsRow = true;
    if (label === 'Data') current.hasDataRow = true;
    for (const field of parseFieldRefs(cell)) {
      (label === 'Topics' ? current.topics : current.data).push({ name: field, line: index + 1 });
    }
  });

  return events;
}

/**
 * Compares a Rust source string against a docs/events.md string and returns
 * every drift between them.
 */
export function checkSources({ rustSource, docsSource }) {
  const codeEvents = parseCodeEvents(rustSource);
  const docEvents = parseDocEvents(docsSource);

  // Duplicates are counted inside each file, not across them: a name appearing
  // in both `src/types.rs` and `docs/events.md` is the point, not a clash.
  const duplicates = [];
  for (const events of [codeEvents, docEvents]) {
    const seen = new Set();
    for (const event of events) {
      if (seen.has(event.name)) duplicates.push(event.name);
      seen.add(event.name);
    }
  }

  const codeByName = new Map();
  for (const event of codeEvents) if (!codeByName.has(event.name)) codeByName.set(event.name, event);
  const docByName = new Map();
  for (const event of docEvents) if (!docByName.has(event.name)) docByName.set(event.name, event);

  const missingInDoc = codeEvents.filter((event) => !docByName.has(event.name));
  const missingInCode = docEvents.filter((event) => !codeByName.has(event.name));
  const missingRows = [];
  const fieldDrift = [];

  for (const [name, codeEvent] of codeByName) {
    const docEvent = docByName.get(name);
    if (docEvent === undefined) continue;

    if (!docEvent.hasTopicsRow) missingRows.push({ name, label: 'Topics', line: docEvent.line });
    if (!docEvent.hasDataRow) missingRows.push({ name, label: 'Data', line: docEvent.line });

    for (const [kind, codeFields, docFields] of [
      ['topics', codeEvent.topics, docEvent.topics],
      ['data', codeEvent.data, docEvent.data],
    ]) {
      const docNames = new Set(docFields.map((field) => field.name));
      const codeNames = new Set(codeFields.map((field) => field.name));
      const inCodeNotDoc = codeFields.filter((field) => !docNames.has(field.name));
      const inDocNotCode = docFields.filter((field) => !codeNames.has(field.name));
      if (inCodeNotDoc.length > 0 || inDocNotCode.length > 0) {
        fieldDrift.push({ name, kind, inCodeNotDoc, inDocNotCode });
      }
    }
  }

  return {
    codeEvents,
    docEvents,
    missingInDoc,
    missingInCode,
    missingRows,
    fieldDrift,
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
    result.missingRows.length === 0 &&
    result.fieldDrift.length === 0 &&
    result.duplicates.length === 0
  );
}

/** Formats a `checkSources` result as a diff-style report. */
export function formatReport(result, { codeSource = CODE_SOURCE, docSource = DOC_SOURCE } = {}) {
  if (isInSync(result)) {
    const fields = result.codeEvents.reduce(
      (total, event) => total + event.topics.length + event.data.length,
      0,
    );
    return `${docSource} is in sync with ${codeSource} (${result.codeEvents.length} events, ${fields} fields checked)`;
  }

  const lines = [`${docSource} is out of sync with ${codeSource}`, ''];

  if (result.missingInDoc.length > 0) {
    lines.push(`  Events in code, missing from ${docSource}:`);
    for (const event of result.missingInDoc) {
      lines.push(`    + ${event.name}  (${codeSource}:${event.line})`);
    }
    lines.push('');
  }

  if (result.missingInCode.length > 0) {
    lines.push(`  Events in ${docSource}, missing from the code:`);
    for (const event of result.missingInCode) {
      lines.push(`    - ${event.name}  (${docSource}:${event.line})`);
    }
    lines.push('');
  }

  if (result.fieldDrift.length > 0) {
    lines.push('  Fields that do not match between the two files:');
    for (const drift of result.fieldDrift) {
      for (const field of drift.inCodeNotDoc) {
        lines.push(
          `    + ${drift.name}.${drift.kind}: ${field.name} is in the code, missing from the ${docSource} ${drift.kind} row` +
            `  (${codeSource}:${field.line})`,
        );
      }
      for (const field of drift.inDocNotCode) {
        lines.push(
          `    - ${drift.name}.${drift.kind}: ${field.name} is in the ${docSource} ${drift.kind} row, missing from the code` +
            `  (${docSource}:${field.line})`,
        );
      }
    }
    lines.push('');
  }

  if (result.missingRows.length > 0) {
    lines.push(`  Sections with no table row to read fields from:`);
    for (const row of result.missingRows) {
      lines.push(`    ! ${row.name}: no ${row.label} row  (${docSource}:${row.line})`);
    }
    lines.push('');
  }

  if (result.duplicates.length > 0) {
    lines.push('  Events documented more than once:');
    for (const name of result.duplicates) {
      lines.push(`    ! ${name}`);
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
    console.error(`check-events: ${error.message}`);
    return 1;
  }
}

const invokedDirectly =
  process.argv[1] !== undefined &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);

if (invokedDirectly) {
  process.exitCode = main();
}
