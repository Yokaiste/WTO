import { applyYsmBranding } from './brand-ysm.ts';

const branding = {
  displayName: 'Custom overhaul',
  menuTitle: 'Custom overhaul + YSM',
  menuByline: 'Custom balance with YSM content',
};

export default function test() {
  const source = [
    '"TOKEN";"REFTEXT"',
    '"YSM";"Yokaiste’s Sandbox Mod"',
    '"YSMTIT";"Yokaiste’s Sandbox Mod 2.4 [Open Beta]"',
    '"YSMMT";"Made with ♥ by Yokaiste"',
    '"YSML1";"Stable version"',
    '',
  ].join('\r\n');
  const branded = applyYsmBranding(source, branding);
  const unchanged = applyYsmBranding('"TOKEN";"REFTEXT"\n"BASE";"WARNO"\n', branding);
  const failures = [
    branded.matchedRows === 3 ? undefined : `Expected 3 replaced rows, got ${branded.matchedRows}.`,
    branded.matchedTokens === 3
      ? undefined
      : `Expected 3 unique tokens, got ${branded.matchedTokens}.`,
    branded.content.includes('"YSM";"Custom overhaul"')
      ? undefined
      : 'The displayed mod name was not replaced.',
    branded.content.includes('"YSMTIT";"Custom overhaul + YSM"')
      ? undefined
      : 'The menu title was not replaced.',
    branded.content.includes('"YSMMT";"Custom balance with YSM content"')
      ? undefined
      : 'The menu byline was not replaced.',
    branded.content.includes('"YSML1";"Stable version"')
      ? undefined
      : 'An unrelated YSM localisation row changed.',
    branded.content.includes('\r\n') ? undefined : 'CRLF line endings were not preserved.',
    unchanged.content === '"TOKEN";"REFTEXT"\n"BASE";"WARNO"\n' && unchanged.matchedRows === 0
      ? undefined
      : 'Content without YSM branding rows must remain unchanged.',
  ].filter((failure): failure is string => failure !== undefined);

  return {
    results: [
      failures.length === 0
        ? {
            name: 'WTO branding changes only the YSM menu identity rows',
            status: 'passed' as const,
          }
        : {
            name: 'WTO branding changes only the YSM menu identity rows',
            status: 'failed' as const,
            reason: 'The conditional localisation rewrite changed the wrong rows or formatting.',
            suggestion: 'Keep the rewrite token-based and preserve all unrelated CSV content.',
            details: failures,
          },
    ],
  };
}
