import type { BuildScriptContext, BuildScriptNdfBlock, GeneratedScriptFile } from 'ymb/api';
import ammunitionRules from '../../ammunition/rules.json';
import missileRules from '../../ammunition-missiles/rules.json';
import flightRules from '../../missiles/rules.json';

interface Rule {
  match: unknown;
  edits: Array<Record<string, unknown>>;
}

interface SourceRules {
  file: string;
  rules: Rule[];
  byEffect: Record<string, Rule>;
}

async function patchObjects(
  context: BuildScriptContext,
  file: string,
  content: string,
  selected: Array<{ block: BuildScriptNdfBlock; rules: Rule[] }>,
): Promise<string> {
  const { ndf } = context.tools;
  const candidates = selected.flatMap(({ block, rules }) => {
    const body = ndf.extractBody(block.text);
    return body ? [{ block, rules, body }] : [];
  });
  if (candidates.length === 0) return content;
  const text = candidates
    .map(({ block, body }, index) => `Subject${index} is ${block.typeName}\n${body.text}`)
    .join('\n');
  const groups = new Map<Rule, string[]>();
  for (const [index, candidate] of candidates.entries())
    for (const rule of new Set(candidate.rules)) {
      const names = groups.get(rule) ?? [];
      names.push(`Subject${index}`);
      groups.set(rule, names);
    }
  const patched = await context.tools.patch(text, {
    file,
    operations: [...groups].map(([rule, names]) => ({
      op: 'bulk',
      match: { conditions: [{ on: 'name', is: 'equals', value: names }] },
      expect: { minBlocks: names.length },
      edits: rule.edits.map(({ minChanges: _minimum, ...edit }) => edit),
    })),
  });
  const changed = ndf.findTopLevelBlocks(patched);
  const parts: string[] = [];
  let cursor = 0;
  for (const [index, candidate] of candidates.entries()) {
    const changedBlock = changed[index];
    const body = changedBlock && ndf.extractBody(changedBlock.text);
    const start = candidate.block.start + candidate.body.start;
    const end = candidate.block.start + candidate.body.end;
    context.tools.assert.ok(body && start >= cursor && changedBlock?.name === `Subject${index}`, {
      reason: 'WTO lost a custom object or selected overlapping objects during patching.',
      suggestion: 'Check the custom ammunition structure and compatibility classification.',
    });
    parts.push(content.slice(cursor, start), body.text);
    cursor = end;
  }
  parts.push(content.slice(cursor));
  return parts.join('');
}

const rangeCategories = new Set([
  'AutocanonHE',
  'CanonAP',
  'DCA',
  'MMG_HMG',
  'LAW',
  'Assault_rifle',
  'SMG',
  'Gatling',
  'Grenade',
  'inf_MMG',
  'Roquette_AS',
  'inf_sniper',
]);
const speedCategories = new Set([
  'AutocanonHE',
  'CanonAP',
  'DCA',
  'MMG_HMG',
  'LAW',
  'Assault_rifle',
  'inf_MMG',
]);

export function categoryName(
  raw: string | undefined,
  definitions: Record<string, number>,
): string | undefined {
  if (!raw) return undefined;
  const symbolic = raw.replace(/^.*\//, '');
  if (symbolic.startsWith('MinMax_') && Object.hasOwn(definitions, symbolic))
    return symbolic.slice(7);
  const number = Number(raw);
  if (!Number.isFinite(number)) return undefined;
  const names = Object.keys(definitions).filter(
    (name) => definitions[name] === number && name.startsWith('MinMax_'),
  );
  return names.length === 1 ? names[0]?.slice(7) : undefined;
}

export function genericTunings(
  category: string | undefined,
  missile: boolean,
  indirect: boolean,
): string[] {
  if (missile) return ['range', ...(category === 'SAM' ? ['aaCycle'] : [])];
  const tunings: string[] = [];
  if (!indirect && category && rangeCategories.has(category)) tunings.push('range');
  if (!indirect && category && speedCategories.has(category)) tunings.push('projectileSpeed');
  if (category === 'obusier') tunings.push('spread');
  if (category && ['obusier', 'Mortier', 'MLRS'].includes(category)) tunings.push('suppression');
  if (category === 'DCA' || category === 'Gatling') tunings.push('aaCycle');
  return tunings;
}

function sourceRules(file: string, rules: Record<string, Rule>): SourceRules {
  return { file, rules: Object.values(rules), byEffect: rules };
}

export default async function extendAmmunition(
  context: BuildScriptContext,
  rules = {
    ammunition: ammunitionRules as Record<string, Rule>,
    missiles: missileRules as Record<string, Rule>,
    flight: flightRules as Record<string, Rule>,
  },
): Promise<GeneratedScriptFile[]> {
  const { ndf } = context.tools;
  const gfx = 'GameData/Generated/Gameplay/Gfx';
  const files = [
    'Ammunition.ndf',
    'AmmunitionMissiles.ndf',
    'WeaponDescriptor.ndf',
    'MissileDescriptors.ndf',
  ].map((file) => `${gfx}/${file}`);
  const ammunition = sourceRules(files[0] as string, rules.ammunition);
  const missiles = sourceRules(files[1] as string, rules.missiles);
  const flight = sourceRules(files[3] as string, rules.flight);
  const sources = [ammunition, missiles];
  const definitions = ndf.readNumericDefinitions(
    await context.readTarget(`${gfx}/Enums/WeaponsMinMaxCategory.ndf`),
  );
  const categories = new Map<string, string | undefined>();
  const readCategory = (raw: string | undefined) => {
    if (raw === undefined) return undefined;
    if (!categories.has(raw)) categories.set(raw, categoryName(raw, definitions));
    return categories.get(raw);
  };
  const originals = await context.readTargets(files);
  const contents = { ...originals };
  const namedRules = sources.flatMap((source) =>
    source.rules
      .filter((rule) => {
        const match = context.tools.values.record(rule.match, 'match');
        return (
          Array.isArray(match.conditions) &&
          match.conditions.some(
            (condition: unknown) =>
              context.tools.values.record(condition, 'condition').on === 'name',
          )
        );
      })
      .map((rule) => ({
        source,
        rule,
        matches: ndf.matcher(rule.match),
      })),
  );
  const flightNames = new Set<string>();
  const output: GeneratedScriptFile[] = [];

  for (const file of files) {
    const content = contents[file] as string;
    const objects = ndf.findObjects(content);
    const protectedRoots = objects.filter((block) =>
      /^(?:WeaponDescriptor|Ammo)_Y[SCNZ]_/.test(block.name ?? ''),
    );
    const candidates: Array<{
      block: BuildScriptNdfBlock;
      rules: Rule[];
    }> = [];
    for (const block of objects) {
      if (
        block.typeName !== 'TAmmunitionDescriptor' ||
        protectedRoots.some((root) => root.start <= block.start && root.end >= block.end)
      )
        continue;
      const clean = ndf.stripComments(block.text);
      const category = readCategory(ndf.readField(clean, 'MinMaxCategory'));
      const missile = ndf.readField(clean, 'MissileDescriptor');
      const isMissile = missile !== undefined && missile !== 'nil';
      if (isMissile && (category === 'ATGM' || category === 'AGM')) {
        if (/^[$~]\//.test(missile)) flightNames.add(missile.slice(missile.lastIndexOf('/') + 1));
      }
      if (block.name?.startsWith('Ammo_') && (file === ammunition.file || file === missiles.file))
        continue;
      const authored = namedRules.filter(({ matches }) => matches(block));
      // Known authored names retain every exception and are never structurally reclassified.
      const selected =
        authored.length > 0
          ? authored
              .filter(({ source }) => source.file !== file || !block.name)
              .map(({ rule }) => rule)
          : genericTunings(
              category,
              isMissile,
              ndf.readField(clean, 'TirIndirect') === 'True',
            ).flatMap((tuning) => {
              if (isMissile && tuning === 'range' && file === missiles.file && block.name)
                return [];
              const source = isMissile ? missiles : ammunition;
              const rule = source.byEffect[tuning];
              context.tools.assert.ok(rule, {
                reason: `WTO cannot find a unique ${tuning} rule.`,
                suggestion:
                  'Update compatibility classification to match the authored WTO rule layout.',
              });
              return [rule];
            });
      if (authored.length > 0 && isMissile && file !== missiles.file)
        selected.push(missiles.byEffect.range as Rule);
      if (selected.length > 0) candidates.push({ block, rules: selected });
    }
    contents[file] = await patchObjects(context, file, content, candidates);
  }

  for (const file of files) {
    const content = contents[file] as string;
    const objects = ndf.findObjects(content);
    const ammoParents = objects.filter(
      (block) =>
        block.typeName === 'TAmmunitionDescriptor' &&
        ['ATGM', 'AGM'].includes(
          readCategory(ndf.readField(ndf.stripComments(block.text), 'MinMaxCategory')) ?? '',
        ),
    );
    const protectedRoots = objects.filter((block) =>
      /^(?:WeaponDescriptor|Ammo)_Y[SCNZ]_/.test(block.name ?? ''),
    );
    const candidates = objects.filter(
      (block) =>
        block.typeName === 'TEntityDescriptor' &&
        (file !== flight.file || !block.name?.startsWith('Descriptor_Missile_')) &&
        !protectedRoots.some((root) => root.start <= block.start && root.end >= block.end) &&
        ((block.name && flightNames.has(block.name)) ||
          (!block.name &&
            ammoParents.some((parent) => parent.start < block.start && parent.end >= block.end))),
    );
    const result = await patchObjects(
      context,
      file,
      content,
      candidates.map((block) => ({ block, rules: flight.rules })),
    );
    if (result !== originals[file]) output.push({ targetRelativePath: file, content: result });
  }
  return output;
}
