// Tests for scripts/check-errors.mjs. Run with: node --test
//
// Fixtures are inline so the tests stay dependency-free and independent of the
// live ERRORS.md.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  checkRepo,
  checkSources,
  formatReport,
  isInSync,
  parseCodeErrors,
  parseDocErrors,
} from './check-errors.mjs';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const RUST_FIXTURE = `use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // 1-9: Initialization & lookup
    /// The contract has not been initialized yet.
    NotInitialized = 1,
    AlreadyInitialized = 2, // trailing comment is fine
}
`;

const DOC_FIXTURE = `# Fixture Contract — Error Codes

## Categories

| Range | Category |
|---|---|
| 1–9 | Initialization & lookup |

## Initialization & lookup (1–9)

| Code | Variant | Raised by | Trigger | User-facing message | Next action |
|---:|---|---|---|---|---|
| 1 | \`NotInitialized\` | \`admin\` | called before setup | "Not set up yet" | contact the admin |
| 2 | \`AlreadyInitialized\` | \`initialize\` | setup run twice | "Already set up" | none |
`;

const checkedHere = { codeSource: 'fixture types.rs', docSource: 'fixture ERRORS.md' };

test('parses variants and codes from the contracterror enum', () => {
  const entries = parseCodeErrors(RUST_FIXTURE);

  assert.deepEqual(
    entries.map((entry) => [entry.variant, entry.code]),
    [
      ['NotInitialized', 1],
      ['AlreadyInitialized', 2],
    ],
  );
});

test('parses rows from the ERRORS.md table and ignores range rows', () => {
  const rows = parseDocErrors(DOC_FIXTURE);

  assert.deepEqual(
    rows.map((entry) => [entry.variant, entry.code]),
    [
      ['NotInitialized', 1],
      ['AlreadyInitialized', 2],
    ],
  );
});

test('matching files pass', () => {
  const result = checkSources({ rustSource: RUST_FIXTURE, docsSource: DOC_FIXTURE });

  assert.equal(isInSync(result), true);
  assert.match(formatReport(result, checkedHere), /is in sync/);
});

test('a variant missing from the doc is reported', () => {
  const rustSource = RUST_FIXTURE.replace(
    '    AlreadyInitialized = 2, // trailing comment is fine\n',
    '    AlreadyInitialized = 2,\n    MissingFromDoc = 3,\n',
  );

  const result = checkSources({ rustSource, docsSource: DOC_FIXTURE });

  assert.equal(isInSync(result), false);
  assert.deepEqual(
    result.missingInDoc.map((entry) => entry.variant),
    ['MissingFromDoc'],
  );
  assert.match(formatReport(result, checkedHere), /Variants in code, missing from/);
});

test('a variant missing from the code is reported', () => {
  const docsSource = `${DOC_FIXTURE}| 9 | \`GhostVariant\` | nobody | never | "?" | ignore |\n`;

  const result = checkSources({ rustSource: RUST_FIXTURE, docsSource });

  assert.equal(isInSync(result), false);
  assert.deepEqual(
    result.missingInCode.map((entry) => entry.variant),
    ['GhostVariant'],
  );
  assert.match(formatReport(result, checkedHere), /missing from the code/);
});

test('a code that differs between the files is reported', () => {
  const docsSource = DOC_FIXTURE.replace('| 1 | `NotInitialized`', '| 7 | `NotInitialized`');

  const result = checkSources({ rustSource: RUST_FIXTURE, docsSource });

  assert.equal(isInSync(result), false);
  assert.equal(result.mismatched.length, 1);
  assert.deepEqual(result.mismatched[0], {
    variant: 'NotInitialized',
    code: 1,
    codeLine: 9,
    docCode: 7,
    docLine: 13,
  });
  assert.match(formatReport(result, checkedHere), /code says 1, fixture ERRORS\.md says 7/);
});

test('a variant documented twice is reported', () => {
  const docsSource = `${DOC_FIXTURE}| 1 | \`NotInitialized\` | duplicate row | duplicate | "?" | ignore |\n`;

  const result = checkSources({ rustSource: RUST_FIXTURE, docsSource });

  assert.equal(isInSync(result), false);
  assert.deepEqual(result.duplicates, ['NotInitialized']);
});

test('a variant without an explicit code is a hard error', () => {
  const rustSource = RUST_FIXTURE.replace('    AlreadyInitialized = 2, // trailing comment is fine\n', '    AlreadyInitialized,\n');

  assert.throws(() => parseCodeErrors(rustSource), /has no explicit code/);
});

test('the real repository files are in sync', () => {
  const result = checkRepo(REPO_ROOT);

  assert.equal(isInSync(result), true, formatReport(result));
});
