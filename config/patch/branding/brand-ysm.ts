import type { BuildScriptContext, GeneratedScriptFile } from 'ymb/api';

interface BrandingResult {
  content: string;
  matchedRows: number;
  matchedTokens: number;
}

type BrandingContext = Pick<
  BuildScriptContext,
  'readTarget' | 'resolvePath' | 'tools' | 'variables'
>;

function renderCsvRow(token: string, value: string): string {
  const escapeCsvValue = (part: string) => part.replace(/\r\n|\r|\n/g, '\\n').replaceAll('"', '""');
  return `"${escapeCsvValue(token)}";"${escapeCsvValue(value)}"`;
}

export function applyBranding(
  content: string,
  replacements: ReadonlyMap<string, string>,
): BrandingResult {
  const newline = content.includes('\r\n') ? '\r\n' : '\n';
  const matchedTokens = new Set<string>();
  let matchedRows = 0;
  const lines = content.split(/\r?\n/).map((line) => {
    if (!line.startsWith('"')) return line;
    const separator = line.indexOf('";"');
    if (separator < 0) return line;
    const token = line.slice(1, separator);
    const replacement = replacements.get(token);
    if (replacement === undefined) return line;
    matchedRows += 1;
    matchedTokens.add(token);
    return renderCsvRow(token, replacement);
  });

  return {
    content: lines.join(newline),
    matchedRows,
    matchedTokens: matchedTokens.size,
  };
}

export default async function brandYsm(context: BrandingContext): Promise<GeneratedScriptFile[]> {
  const brandingValue = context.tools.values.record(context.variables.branding, 'branding');
  const replacements = new Map(
    Object.entries(brandingValue).map(([token, value]) => [
      token,
      context.tools.values.string(value, `branding.${token}`),
    ]),
  );
  const modRootName = context.tools.values.string(context.variables.modRootName, 'modRootName');
  const targetRelativePath = `GameData/Localisation/${modRootName}/INTERFACE_OUTGAME.csv`;
  const result = applyBranding(await context.readTarget(targetRelativePath), replacements);
  if (result.matchedRows === 0) return [];

  context.tools.assert.ok(
    result.matchedRows === replacements.size && result.matchedTokens === replacements.size,
    {
      reason:
        'YSM menu localisation was present but its branding rows were incomplete or duplicated.',
      suggestion:
        'Update the WTO branding tokens to match the current YSM welcome-screen localisation contract.',
      details: [
        `Expected rows: ${replacements.size}`,
        `Matched rows: ${result.matchedRows}`,
        `Unique tokens: ${result.matchedTokens}`,
      ],
      absolutePath: context.resolvePath(targetRelativePath),
    },
  );

  return [{ targetRelativePath, content: result.content }];
}
