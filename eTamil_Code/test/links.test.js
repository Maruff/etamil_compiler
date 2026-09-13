// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//
// The documentation links.
//
// A dead link in a QuickPick is worse than no QuickPick: it is offered by the
// editor, so it reads as something the project stands behind. These tests
// cannot tell whether a page still exists — that needs the network, and a test
// suite that fails when a DNS server is slow is a test suite people learn to
// ignore — so they check the things that can be checked offline, and the
// README is held to the same table so the two cannot drift.
//
//   node --test test/links.test.js
//
// Requires `npm run build`; out/links.js imports nothing from the vscode API.

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { test, describe } = require('node:test');

const BUILT = path.join(__dirname, '..', 'out', 'links.js');
const available = fs.existsSync(BUILT);
const links = available ? require(BUILT) : null;

describe('documentation links', { skip: available ? false : 'run npm run build' }, () => {
  test('every entry is complete', () => {
    assert.ok(links.LINKS.length >= 5, `only ${links.LINKS.length} links`);
    for (const link of links.LINKS) {
      assert.ok(link.label, 'a link with no label');
      assert.ok(link.detail, `${link.label} has no detail`);
      assert.ok(link.url, `${link.label} has no url`);
    }
  });

  test('every URL is https, and on a host this project controls', () => {
    for (const { label, url } of links.LINKS) {
      assert.match(url, /^https:\/\//, `${label} is not https`);
      const host = new URL(url).host;
      assert.ok(
        host === 'etamil.in' || host === 'github.com',
        `${label} points at ${host}`
      );
    }
  });

  test('a site path ends in a slash, because the site redirects otherwise', () => {
    for (const { label, url } of links.LINKS) {
      if (new URL(url).host !== 'etamil.in') {
        continue;
      }
      const pathname = new URL(url).pathname;
      assert.ok(pathname.endsWith('/'), `${label}: ${pathname} has no trailing slash`);
    }
  });

  test('no two entries send you to the same page', () => {
    const urls = links.LINKS.map((link) => link.url);
    assert.equal(new Set(urls).size, urls.length);
  });

  test('the manual and the keyword reference are in the list', () => {
    const urls = links.LINKS.map((link) => link.url);
    assert.ok(urls.includes(links.MANUAL), 'MANUAL is not one of the links');
    assert.ok(
      urls.includes(links.KEYWORDS_REFERENCE),
      'KEYWORDS_REFERENCE is not one of the links'
    );
  });

  test('the README points at the same manual the command does', () => {
    // Two places telling somebody where the manual is, and one of them wrong,
    // is the ordinary way a link rots. They are checked against each other.
    const readme = fs.readFileSync(path.join(__dirname, '..', 'README.md'), 'utf8');
    assert.ok(readme.includes(links.MANUAL), `README.md does not link ${links.MANUAL}`);
  });
});
