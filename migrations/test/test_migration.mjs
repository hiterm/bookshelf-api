#!/usr/bin/env node
// Migration test: verifies that the add_change_history migration correctly
// creates snapshot events for all existing books and authors.
//
// Requires two empty PostgreSQL databases to be created before running:
//   MIGRATION_TEST_EMPTY_URL  - for the empty-DB scenario
//   MIGRATION_TEST_DATA_URL   - for the data scenario
//
// Usage:
//   MIGRATION_TEST_EMPTY_URL=postgres://... \
//   MIGRATION_TEST_DATA_URL=postgres://... \
//   node migrations/test/test_migration.mjs

import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const MIGRATIONS_DIR = join(dirname(fileURLToPath(import.meta.url)), '..');

const EMPTY_URL = process.env.MIGRATION_TEST_EMPTY_URL;
const DATA_URL = process.env.MIGRATION_TEST_DATA_URL;

if (!EMPTY_URL || !DATA_URL) {
  console.error('MIGRATION_TEST_EMPTY_URL and MIGRATION_TEST_DATA_URL must be set');
  process.exit(1);
}

// Run SQL against the given database URL via stdin.
function psql(url, sql) {
  execSync(`psql --set ON_ERROR_STOP=1 ${JSON.stringify(url)}`, {
    input: sql,
    stdio: ['pipe', 'pipe', 'inherit'],
  });
}

// Run a single-column SELECT and return the trimmed scalar result.
function queryOne(url, sql) {
  return execSync(`psql -t -A ${JSON.stringify(url)}`, {
    input: sql,
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe'],
  }).trim();
}

function applyMigration(url, filename) {
  psql(url, readFileSync(join(MIGRATIONS_DIR, filename), 'utf8'));
}

// ---- Test runner ----

let passed = 0;
let failed = 0;

function test(name, fn) {
  try {
    fn();
    console.log(`[PASS] ${name}`);
    passed++;
  } catch (err) {
    console.error(`[FAIL] ${name}`);
    console.error(`       ${err.message}`);
    failed++;
  }
}

function assertEqual(actual, expected, label) {
  if (actual !== String(expected)) {
    throw new Error(`${label}: expected ${JSON.stringify(String(expected))}, got ${JSON.stringify(actual)}`);
  }
}

// ---- Setup ----

console.log('Applying base schema...');
applyMigration(EMPTY_URL, '20220306122339_create_tables.sql');
applyMigration(DATA_URL, '20220306122339_create_tables.sql');

console.log('Inserting test data...');
psql(DATA_URL, `
  INSERT INTO bookshelf_user (id) VALUES
    ('user_alpha'),
    ('user_beta'),
    ('user_gamma');  -- user_gamma: no books or authors

  INSERT INTO book (id, user_id, title, isbn, read, owned, priority, format, store) VALUES
    ('a0000000-0000-0000-0000-000000000001', 'user_alpha', 'Book A1', 'ISBN-A1', false, true,  0, 'Unknown', 'Unknown'),
    ('a0000000-0000-0000-0000-000000000002', 'user_alpha', 'Book A2', 'ISBN-A2', true,  false, 1, 'eBook',   'Kindle'),
    ('b0000000-0000-0000-0000-000000000001', 'user_beta',  'Book B1', 'ISBN-B1', false, false, 0, 'Unknown', 'Unknown');

  INSERT INTO author (id, user_id, name, yomi) VALUES
    ('a1000000-0000-0000-0000-000000000001', 'user_alpha', 'Author A', 'authora'),
    ('b1000000-0000-0000-0000-000000000001', 'user_beta',  'Author B', 'authorb');

  -- user_alpha Book A1 and A2 both written by Author A
  INSERT INTO book_author (user_id, book_id, author_id) VALUES
    ('user_alpha', 'a0000000-0000-0000-0000-000000000001', 'a1000000-0000-0000-0000-000000000001'),
    ('user_alpha', 'a0000000-0000-0000-0000-000000000002', 'a1000000-0000-0000-0000-000000000001');
`);

console.log('Applying change-history migration...');
applyMigration(EMPTY_URL, '20260429040611_add_change_history.sql');
applyMigration(DATA_URL, '20260429040611_add_change_history.sql');

// ---- Empty-DB scenario ----

console.log('\n-- empty DB --');

test('migration runs without error on empty DB', () => {
  // Reaching here means applyMigration(EMPTY_URL, ...) above succeeded.
});

test('empty DB: no event_sets created', () => {
  assertEqual(queryOne(EMPTY_URL, 'SELECT COUNT(*) FROM event_set'), 0, 'event_set count');
});

test('empty DB: no book_events created', () => {
  assertEqual(queryOne(EMPTY_URL, 'SELECT COUNT(*) FROM book_event'), 0, 'book_event count');
});

test('empty DB: no author_events created', () => {
  assertEqual(queryOne(EMPTY_URL, 'SELECT COUNT(*) FROM author_event'), 0, 'author_event count');
});

// ---- Data scenario: counts ----

console.log('\n-- data DB: counts --');

test('snapshot_all event_set created for each user that has data (2 of 3 users)', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT COUNT(*) FROM event_set WHERE operation = 'snapshot_all'"),
    2, 'snapshot_all event_set count',
  );
});

test('user_gamma (no books or authors) has no event_set', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT COUNT(*) FROM event_set WHERE user_id = 'user_gamma'"),
    0, 'user_gamma event_set count',
  );
});

test('3 book snapshot events (one per book)', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT COUNT(*) FROM book_event WHERE operation = 'snapshot'"),
    3, 'book_event snapshot count',
  );
});

test('2 author snapshot events (one per author)', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT COUNT(*) FROM author_event WHERE operation = 'snapshot'"),
    2, 'author_event snapshot count',
  );
});

test('2 book_event_author rows (matching source book_author)', () => {
  assertEqual(
    queryOne(DATA_URL, 'SELECT COUNT(*) FROM book_event_author'),
    2, 'book_event_author count',
  );
});

// ---- Data scenario: snapshot content ----

console.log('\n-- data DB: snapshot content --');

test('book_event.title matches source book', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT title FROM book_event WHERE book_id = 'a0000000-0000-0000-0000-000000000001'"),
    'Book A1', 'Book A1 title',
  );
});

test('book_event.isbn matches source book', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT isbn FROM book_event WHERE book_id = 'a0000000-0000-0000-0000-000000000002'"),
    'ISBN-A2', 'Book A2 isbn',
  );
});

test('book_event.read matches source book (Book A2: true)', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT read FROM book_event WHERE book_id = 'a0000000-0000-0000-0000-000000000002'"),
    't', 'Book A2 read',
  );
});

test('book_event.format and store match source book (Book A2: eBook/Kindle)', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT format FROM book_event WHERE book_id = 'a0000000-0000-0000-0000-000000000002'"),
    'eBook', 'Book A2 format',
  );
  assertEqual(
    queryOne(DATA_URL, "SELECT store FROM book_event WHERE book_id = 'a0000000-0000-0000-0000-000000000002'"),
    'Kindle', 'Book A2 store',
  );
});

test('book_event.priority matches source book (Book A2: 1)', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT priority FROM book_event WHERE book_id = 'a0000000-0000-0000-0000-000000000002'"),
    1, 'Book A2 priority',
  );
});

test('book_event.user_id matches source book', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT user_id FROM book_event WHERE book_id = 'b0000000-0000-0000-0000-000000000001'"),
    'user_beta', 'Book B1 user_id',
  );
});

test('book_event.book_created_at and book_updated_at are not NULL', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT COUNT(*) FROM book_event WHERE operation = 'snapshot' AND book_created_at IS NULL"),
    0, 'NULL book_created_at count',
  );
  assertEqual(
    queryOne(DATA_URL, "SELECT COUNT(*) FROM book_event WHERE operation = 'snapshot' AND book_updated_at IS NULL"),
    0, 'NULL book_updated_at count',
  );
});

test('author_event.name matches source author', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT name FROM author_event WHERE author_id = 'a1000000-0000-0000-0000-000000000001'"),
    'Author A', 'Author A name',
  );
});

test('author_event.yomi matches source author', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT yomi FROM author_event WHERE author_id = 'a1000000-0000-0000-0000-000000000001'"),
    'authora', 'Author A yomi',
  );
});

test('author_event.author_created_at is not NULL', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT COUNT(*) FROM author_event WHERE operation = 'snapshot' AND author_created_at IS NULL"),
    0, 'NULL author_created_at count',
  );
});

test('book_event_author links Book A1 to Author A', () => {
  assertEqual(
    queryOne(DATA_URL, `
      SELECT COUNT(*) FROM book_event_author bea
      JOIN book_event be ON bea.event_id = be.event_id
      WHERE be.book_id = 'a0000000-0000-0000-0000-000000000001'
        AND bea.author_id = 'a1000000-0000-0000-0000-000000000001'
    `),
    1, 'Book A1 -> Author A link',
  );
});

test('book_event_author links Book A2 to Author A', () => {
  assertEqual(
    queryOne(DATA_URL, `
      SELECT COUNT(*) FROM book_event_author bea
      JOIN book_event be ON bea.event_id = be.event_id
      WHERE be.book_id = 'a0000000-0000-0000-0000-000000000002'
        AND bea.author_id = 'a1000000-0000-0000-0000-000000000001'
    `),
    1, 'Book A2 -> Author A link',
  );
});

test('extra is NULL for all snapshot events', () => {
  assertEqual(
    queryOne(DATA_URL, "SELECT COUNT(*) FROM book_event WHERE operation = 'snapshot' AND extra IS NOT NULL"),
    0, 'non-NULL extra in book snapshots',
  );
  assertEqual(
    queryOne(DATA_URL, "SELECT COUNT(*) FROM author_event WHERE operation = 'snapshot' AND extra IS NOT NULL"),
    0, 'non-NULL extra in author snapshots',
  );
});

test('each snapshot event_set is linked to the correct user', () => {
  assertEqual(
    queryOne(DATA_URL, `
      SELECT COUNT(*) FROM book_event be
      JOIN event_set es ON be.event_set_id = es.id
      WHERE es.user_id = 'user_alpha' AND be.operation = 'snapshot'
    `),
    2, 'user_alpha book snapshot event count',
  );
  assertEqual(
    queryOne(DATA_URL, `
      SELECT COUNT(*) FROM book_event be
      JOIN event_set es ON be.event_set_id = es.id
      WHERE es.user_id = 'user_beta' AND be.operation = 'snapshot'
    `),
    1, 'user_beta book snapshot event count',
  );
});

// ---- Summary ----

console.log(`\n${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
