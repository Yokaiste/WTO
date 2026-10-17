import assert from 'node:assert/strict';
import type { BuildScriptTest } from 'ymb/api';
import brandMenu, { applyBranding } from './brand-ysm.ts';

const test: BuildScriptTest = async ({ tools }) => {
  const replacements = new Map([
    ['TITLE_A', 'A new title'],
    ['TITLE_B', 'Line 1\r\n"Line 2"'],
  ]);
  const source =
    '"TOKEN";"REFTEXT"\r\n"TITLE_A";"Before"\r\n"TITLE_B";"Before"\r\n"OTHER";"Keep me"\r\n';
  const result = applyBranding(source, replacements);
  assert.equal(result.matchedRows, replacements.size);
  assert.equal(result.matchedTokens, replacements.size);
  assert.ok(result.content.includes('"TITLE_A";"A new title"\r\n'));
  assert.ok(result.content.includes('"TITLE_B";"Line 1\\n""Line 2"""\r\n'));
  assert.ok(result.content.includes('"OTHER";"Keep me"\r\n'));
  assert.deepEqual(applyBranding(source, new Map()), {
    content: source,
    matchedRows: 0,
    matchedTokens: 0,
  });
  assert.equal(applyBranding(result.content, replacements).content, result.content);
  const context = {
    tools,
    variables: { branding: Object.fromEntries(replacements), modRootName: 'sample' },
    resolvePath: (value: string) => value,
    readTarget: async () => source,
  };
  assert.equal((await brandMenu(context))[0]?.content, result.content);
  assert.deepEqual(await brandMenu({ ...context, readTarget: async () => '"OTHER";"Keep"\n' }), []);
  for (const invalid of [
    source.replace('"TITLE_B"', '"ABSENT"'),
    `${source}"TITLE_A";"Duplicate"\r\n`,
  ]) {
    await assert.rejects(() => brandMenu({ ...context, readTarget: async () => invalid }));
  }
  return { results: [{ name: 'Configurable localisation replacement', status: 'passed' }] };
};
export default test;
