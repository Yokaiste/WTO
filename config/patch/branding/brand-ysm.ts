import type { BuildScriptContext, GeneratedScriptFile } from 'ymb/api';

export interface YsmBranding {
  displayName: string;
  menuTitle: string;
  menuByline: string;
}

function brandingRows(branding: YsmBranding): ReadonlyMap<string, string> {
  return new Map([
    ['YSM', branding.displayName],
    ['YSMTIT', branding.menuTitle],
    ['YSMMT', branding.menuByline],
  ]);
}

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
  const escapeCsvValue = (part: string) => part.replaceAll('"', '""');
  return `"${escapeCsvValue(token)}";"${escapeCsvValue(value)}"`;
}

export function applyYsmBranding(content: string, branding: YsmBranding): BrandingResult {
  const replacements = brandingRows(branding);
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
  const branding = {
    displayName: context.tools.values.string(brandingValue.displayName, 'branding.displayName'),
    menuTitle: context.tools.values.string(brandingValue.menuTitle, 'branding.menuTitle'),
    menuByline: context.tools.values.string(brandingValue.menuByline, 'branding.menuByline'),
  };
  const modRootName = context.tools.values.string(context.variables.modRootName, 'modRootName');
  const targetRelativePath = `GameData/Localisation/${modRootName}/INTERFACE_OUTGAME.csv`;
  const result = applyYsmBranding(await context.readTarget(targetRelativePath), branding);
  if (result.matchedRows === 0) return [];

  context.tools.assert.ok(result.matchedRows === 3 && result.matchedTokens === 3, {
    reason:
      'YSM menu localisation was present but its branding rows were incomplete or duplicated.',
    suggestion:
      'Update the WTO branding tokens to match the current YSM welcome-screen localisation contract.',
    details: [
      'Expected rows: 3',
      `Matched rows: ${result.matchedRows}`,
      `Unique tokens: ${result.matchedTokens}`,
    ],
    absolutePath: context.resolvePath(targetRelativePath),
  });

  return [{ targetRelativePath, content: result.content }];
}
