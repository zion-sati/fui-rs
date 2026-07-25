import assert from 'node:assert/strict';
import test from 'node:test';

import { classifyCargoPublish } from './publish-crate-idempotently.mjs';

test('accepts a successful Cargo publication', () => {
  assert.equal(classifyCargoPublish(0, '', 'fui-rs', '0.2.3-alpha1'), 'published');
});

test('accepts only the exact duplicate crate version as an idempotent replay', () => {
  const output = 'error: crate fui-rs@0.2.3-alpha1 already exists on crates.io index';
  assert.equal(classifyCargoPublish(101, output, 'fui-rs', '0.2.3-alpha1'), 'already-published');
  assert.equal(classifyCargoPublish(101, output, 'fui-rs', '0.2.3-alpha2'), 'failed');
  assert.equal(classifyCargoPublish(101, output, 'another-crate', '0.2.3-alpha1'), 'failed');
});

test('preserves unrelated Cargo and registry failures', () => {
  assert.equal(classifyCargoPublish(101, 'error: failed to verify package', 'fui-rs', '0.2.3-alpha1'), 'failed');
  assert.equal(classifyCargoPublish(null, 'spawn cargo ENOENT', 'fui-rs', '0.2.3-alpha1'), 'failed');
});
