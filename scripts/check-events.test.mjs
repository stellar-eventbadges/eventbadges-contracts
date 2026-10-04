// Tests for scripts/check-events.mjs. Run with: node --test
//
// Fixtures are inline so the tests stay dependency-free and independent of the
// live docs/events.md.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  checkRepo,
  checkSources,
  formatReport,
  isInSync,
  parseCodeEvents,
  parseDocEvents,
} from './check-events.mjs';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const RUST_FIXTURE = `use soroban_sdk::{contractevent, Address};

#[contractevent]
#[derive(Clone, Debug)]
pub struct ThingDone {
    /// The thing's id.
    #[topic]
    pub thing_id: u64,
    /// Who did it.
    #[topic]
    pub actor: Address,
    /// When it happened.
    pub at: u64,
}
`;

const DOC_FIXTURE = `# Fixture Contract — Events

## \`ThingDone\`

Emitted once per thing.

| | |
|---|---|
| Topics | \`Symbol("thing_done")\`, then \`thing_id\` (\`u64\`) and \`actor\` (\`Address\`) |
| Data | map with \`at\` (\`u64\`) |
| Emitted by | \`do_thing\` in \`src/things.rs\` |
`;

/** The fixture above with one field's type changed in the docs row. */
const DOC_FIXTURE_WRONG_TYPE = DOC_FIXTURE.replace('map with `at` (`u64`)', 'map with `at` (`u32`)');

const checkedHere = { codeSource: 'fixture types.rs', docSource: 'fixture events.md' };

test('parses topic and data fields from contractevent structs', () => {
  const events = parseCodeEvents(RUST_FIXTURE);

  assert.equal(events.length, 1);
  assert.equal(events[0].name, 'ThingDone');
  assert.deepEqual(
    events[0].topics.map((field) => field.name),
    ['thing_id', 'actor'],
  );
  assert.deepEqual(
    events[0].data.map((field) => field.name),
    ['at'],
  );
});

test('parses sections and their Topics and Data rows', () => {
  const events = parseDocEvents(DOC_FIXTURE);

  assert.equal(events.length, 1);
  assert.equal(events[0].name, 'ThingDone');
  assert.deepEqual(
    events[0].topics.map((field) => field.name),
    ['thing_id', 'actor'],
  );
  assert.deepEqual(
    events[0].data.map((field) => field.name),
    ['at'],
  );
});

test('matching files pass', () => {
  const result = checkSources({ rustSource: RUST_FIXTURE, docsSource: DOC_FIXTURE });

  assert.equal(isInSync(result), true);
  assert.match(formatReport(result, checkedHere), /in sync/);
});

test('a field missing from the docs is reported', () => {
  const docsSource = DOC_FIXTURE.replace(' and `actor` (`Address`)', '');
  const result = checkSources({ rustSource: RUST_FIXTURE, docsSource });

  assert.equal(isInSync(result), false);
  assert.deepEqual(
    result.fieldDrift.map((drift) => [drift.name, drift.kind, drift.inCodeNotDoc.map((f) => f.name)]),
    [['ThingDone', 'topics', ['actor']]],
  );
  assert.match(formatReport(result, checkedHere), /ThingDone\.topics: actor is in the code/);
});

test('a field missing from the code is reported', () => {
  const rustSource = RUST_FIXTURE.replace(
    '    /// When it happened.\n    pub at: u64,\n',
    '',
  );
  const result = checkSources({ rustSource, docsSource: DOC_FIXTURE });

  assert.equal(isInSync(result), false);
  assert.deepEqual(
    result.fieldDrift.map((drift) => [drift.name, drift.kind, drift.inDocNotCode.map((f) => f.name)]),
    [['ThingDone', 'data', ['at']]],
  );
});

test('a field written without a type reads as missing, rather than matching silently', () => {
  const docsSource = DOC_FIXTURE.replace('map with `at` (`u64`)', 'map with `at`');
  const result = checkSources({ rustSource: RUST_FIXTURE, docsSource });

  assert.equal(isInSync(result), false);
  assert.deepEqual(
    result.fieldDrift.map((drift) => drift.inCodeNotDoc.map((f) => f.name)),
    [['at']],
  );
});

test('a changed type is not reported, because only field names are compared', () => {
  // The docs row spells the type for the reader, but the check compares names:
  // a type that has drifted is caught by the compiler and the tests in
  // src/test.rs, which pin the exact data map.
  const result = checkSources({ rustSource: RUST_FIXTURE, docsSource: DOC_FIXTURE_WRONG_TYPE });

  assert.equal(isInSync(result), true);
});

test('an event in the code but not the docs is reported', () => {
  const rustSource = `${RUST_FIXTURE}
#[contractevent]
#[derive(Clone, Debug)]
pub struct OtherDone {
    /// Another id.
    #[topic]
    pub other_id: u64,
}
`;
  const result = checkSources({ rustSource, docsSource: DOC_FIXTURE });

  assert.equal(isInSync(result), false);
  assert.deepEqual(
    result.missingInDoc.map((event) => event.name),
    ['OtherDone'],
  );
  assert.match(formatReport(result, checkedHere), /OtherDone/);
});

test('a documented event with no struct is reported', () => {
  const docsSource = `${DOC_FIXTURE}\n## \`GhostDone\`\n\n| | |\n|---|---|\n| Topics | \`ghost_id\` (\`u64\`) |\n| Data | map with \`at\` (\`u64\`) |\n`;
  const result = checkSources({ rustSource: RUST_FIXTURE, docsSource });

  assert.equal(isInSync(result), false);
  assert.deepEqual(
    result.missingInCode.map((event) => event.name),
    ['GhostDone'],
  );
  assert.match(formatReport(result, checkedHere), /GhostDone/);
});

test('a section with no Data row is reported', () => {
  const docsSource = DOC_FIXTURE.replace('| Data | map with `at` (`u64`) |\n', '');
  const result = checkSources({ rustSource: RUST_FIXTURE, docsSource });

  assert.equal(isInSync(result), false);
  assert.deepEqual(
    result.missingRows.map((row) => [row.name, row.label]),
    [['ThingDone', 'Data']],
  );
});

test('a struct with no contractevent attribute is ignored', () => {
  const rustSource = `${RUST_FIXTURE}\n#[derive(Clone)]\npub struct NotAnEvent {\n    pub field: u64,\n}\n`;
  const events = parseCodeEvents(rustSource);

  assert.deepEqual(
    events.map((event) => event.name),
    ['ThingDone'],
  );
});

test('a duplicate struct is reported once', () => {
  const rustSource = `${RUST_FIXTURE}\n${RUST_FIXTURE}`;
  const result = checkSources({ rustSource, docsSource: DOC_FIXTURE });

  assert.equal(isInSync(result), false);
  assert.deepEqual(result.duplicates, ['ThingDone']);
});

test('a source with no contractevent structs is a hard error', () => {
  assert.throws(() => parseCodeEvents('pub struct Nothing {}'), /no #\[contractevent\] structs/);
});

test('the real repository files are in sync', () => {
  const result = checkRepo(REPO_ROOT);

  assert.equal(isInSync(result), true);
  assert.deepEqual(
    result.codeEvents.map((event) => event.name),
    ['EventCreated', 'BadgeClaimed', 'BadgeAwarded', 'BadgeRevoked'],
  );
});
