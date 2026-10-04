// Execute real library output in a pinned terminal emulator. Unlike a PTY,
// this observer owns the soft-wrap markers and can test retained scrollback.
const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const path = require('node:path');
const { Terminal } = require('@xterm/headless');
const root = path.resolve(__dirname, '../..');
execFileSync('cargo', ['build', '--locked', '--example', 'flow'], { cwd: root, stdio: 'inherit' });
const binary = path.join(process.env.CARGO_TARGET_DIR || path.join(root, 'target'), 'debug', 'examples', 'flow');
function output(args) {
  return execFileSync(binary, args, { env: { ...process.env, NO_COLOR: '1' }, encoding: 'utf8' });
}
function logicalLines(terminal) {
  const buffer = terminal.buffer.active;
  const lines = [];
  for (let row = 0; row < buffer.length; row++) {
    const line = buffer.getLine(row);
    const value = line.translateToString(!buffer.getLine(row + 1)?.isWrapped);
    if (line.isWrapped) lines[lines.length - 1] += value;
    else lines.push(value);
  }
  // Ignore emulator padding below the final transcript line after resize.
  while (lines.length && lines.at(-1).trim() === '') lines.pop();
  return lines;
}
async function observe(text) {
  const terminal = new Terminal({ cols: 20, rows: 4, scrollback: 1000, allowProposedApi: true });
  // Match normal POSIX output processing. Keep enough following text to move
  // the paragraph into scrollback, away from the emulator's active cursor row.
  const transcript = text + 'tail one\ntail two\ntail three\ntail four\n';
  await new Promise(resolve => terminal.write(transcript.replaceAll('\n', '\r\n'), resolve));
  const original = logicalLines(terminal);
  const lengths = [];
  for (const columns of [60, 12, 100, 20]) {
    terminal.resize(columns, 4);
    assert.deepEqual(logicalLines(terminal), original, `logical content changed at ${columns}`);
    lengths.push(terminal.buffer.active.length);
  }
  terminal.dispose();
  return { original, lengths };
}
(async () => {
  const flow = output([]);
  assert.equal(flow.split('\n').length, 3, 'document injected physical line breaks');
  const observed = await observe(flow);
  assert.equal(observed.original[0], flow.split('\n')[0]);
  assert(observed.lengths[1] > observed.lengths[0], 'narrowing did not reflow already emitted text');
  assert(observed.lengths[2] < observed.lengths[1], 'widening did not rejoin soft wraps');
  const fragments = output(['--fragments']);
  assert.equal(fragments.split('\n').length, 3, 'fragment encoding injected line breaks');
  await observe(fragments);
  const live = new Terminal({ cols: 20, rows: 4, scrollback: 1000, allowProposedApi: true });
  await new Promise(resolve => live.write('before resize\r\n', resolve));
  live.resize(60, 4);
  await new Promise(resolve => live.write(fragments.replaceAll('\n', '\r\n'), resolve));
  assert.deepEqual(logicalLines(live), ['before resize', ...fragments.trimEnd().split('\n')], 'new output used stale geometry');
  // xterm 5.5.0 intentionally skips the paragraph containing the cursor during
  // reflow. Record that boundary instead of claiming application-owned replay.
  live.dispose();
  const fixed = await observe(output(['--fixed']));
  assert(fixed.original.length > observed.original.length, 'negative control lost hard line breaks');
  console.log('PASS xterm/headless 5.5.0: document/fragment scrollback at 20→60→12→100→20; fixed-layout negative control');
})().catch(error => { console.error(error); process.exitCode = 1; });
