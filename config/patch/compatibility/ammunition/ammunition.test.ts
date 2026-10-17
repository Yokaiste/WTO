import type { BuildScriptContext } from 'ymb/api';
import extendAmmunition, { categoryName, genericTunings } from './ammunition.ts';

export default async function test(context: BuildScriptContext) {
  const gfx = 'GameData/Generated/Gameplay/Gfx';
  const multipliers: Record<string, number> = {
    ammunitionRangeMultiplier: 2,
    projectileSpeedMultiplier: 1.5,
    missileRangeMultiplier: 2,
    aaLockTimeMultiplier: 3,
    missileFlightMultiplier: 1.5,
  };
  const rule = (field: string, tuning: string) => ({
    op: 'bulk',
    match: { conditions: [{ on: 'type', is: 'equals', value: 'TAmmunitionDescriptor' }] },
    edits: [{ field, multiply: `\${tuning.${tuning}}` }],
  });
  const rules = {
    ammunition: {
      range: rule('MaximumRangeGRU', 'ammunitionRangeMultiplier'),
      projectileSpeed: rule('ProjectileSpeedGRU', 'projectileSpeedMultiplier'),
    },
    missiles: {
      range: rule('MaximumRangeGRU', 'missileRangeMultiplier'),
      aaCycle: rule('AimingTime', 'aaLockTimeMultiplier'),
    },
    flight: { flight: rule('MaxSpeedGRU', 'missileFlightMultiplier') },
  };
  const definitions = { MinMax_AutocanonHE: 41, MinMax_SAM: 52, MinMax_ATGM: 63 };
  const inputs: Record<string, string> = {
    [`${gfx}/Enums/WeaponsMinMaxCategory.ndf`]: Object.entries(definitions)
      .map(([name, value]) => `${name} is ${value}`)
      .join('\n'),
    [`${gfx}/Ammunition.ndf`]:
      'Ammo_Custom is TAmmunitionDescriptor(MinMaxCategory = 41 MaximumRangeGRU = 100)\nAmmo_Protected is TAmmunitionDescriptor(MinMaxCategory = 63 MissileDescriptor = ~/Descriptor_Missile_Other)',
    [`${gfx}/AmmunitionMissiles.ndf`]:
      'Decooked_SAM is TAmmunitionDescriptor(MinMaxCategory = 52 MaximumRangeGRU = 200 AimingTime = 2 TimeBetweenTwoShots = 3 MissileDescriptor = ~/Missile_SAM)',
    [`${gfx}/MissileDescriptors.ndf`]:
      'CustomFlight is TEntityDescriptor(ModulesDescriptors = [TMovement(MaxSpeedGRU = 100 MaxAccelerationGRU = 20)])\nDescriptor_Missile_Other is TEntityDescriptor(MaxSpeedGRU = 120)',
    [`${gfx}/WeaponDescriptor.ndf`]: `Weapon is TWeaponManagerModuleDescriptor( Ammunition = [
      TAmmunitionDescriptor(MinMaxCategory = 41 // MissileDescriptor is only a comment
        MaximumRangeGRU = 100 ProjectileSpeedGRU = 200 Arme = TDamageTypeRTTI(Family = CustomDamageFamily Index = 5)),
      TAmmunitionDescriptor(MinMaxCategory = 410 MaximumRangeGRU = 100),
      TAmmunitionDescriptor(MinMaxCategory = MinMax_ATGM MaximumRangeGRU = 300 MissileDescriptor = ~/CustomFlight),
      TAmmunitionDescriptor(MinMaxCategory = 63 MaximumRangeGRU = 300 MissileDescriptor = TEntityDescriptor(ModulesDescriptors = [TMovement(MaxSpeedGRU = 100 MaxAccelerationGRU = 20)])),
    ])
    WeaponDescriptor_YC_Illegal is TWeaponManagerModuleDescriptor(Ammunition = TAmmunitionDescriptor(MinMaxCategory = 41 MaximumRangeGRU = 999))
    Decooked_CustomGun is TAmmunitionDescriptor(MinMaxCategory = 41 MaximumRangeGRU = 100 ProjectileSpeedGRU = 200)`,
  };
  const targetReads: string[] = [];
  const output = await extendAmmunition(
    {
      ...context,
      variables: { gfx },
      tools: {
        ...context.tools,
        patch: (text, patch) =>
          context.tools.patch(
            text,
            JSON.parse(
              JSON.stringify(patch).replace(/"\$\{tuning\.(\w+)\}"/g, (_, key) => {
                if (!Object.hasOwn(multipliers, key))
                  throw new Error(`Unknown fixture multiplier: ${key}`);
                return String(multipliers[key]);
              }),
            ),
          ),
      },
      readTarget: async (file) => {
        targetReads.push(file);
        return inputs[file] ?? '';
      },
      readTargets: async (files) =>
        Object.fromEntries(files.map((file) => [file, inputs[file] ?? ''])),
    },
    rules,
  );
  const results = Object.fromEntries(
    output.map((entry) => [entry.targetRelativePath, String(entry.content)]),
  );
  const ndf = context.tools.ndf;
  const weapons = results[`${gfx}/WeaponDescriptor.ndf`] ?? '';
  const ranges = ndf
    .findObjects(weapons)
    .filter((block) => block.typeName === 'TAmmunitionDescriptor')
    .map((block) => ndf.readField(ndf.stripComments(block.text), 'MaximumRangeGRU'));
  context.tools.assert.ok(
    JSON.stringify(ranges) === JSON.stringify(['200', '100', '600', '600', '999', '200']) &&
      !results[`${gfx}/Ammunition.ndf`] &&
      results[`${gfx}/AmmunitionMissiles.ndf`]?.includes('MaximumRangeGRU = 200') &&
      results[`${gfx}/AmmunitionMissiles.ndf`]?.includes('AimingTime = 6') &&
      results[`${gfx}/MissileDescriptors.ndf`]?.includes('MaxSpeedGRU = 150') &&
      results[`${gfx}/MissileDescriptors.ndf`]?.includes(
        'Descriptor_Missile_Other is TEntityDescriptor(MaxSpeedGRU = 120)',
      ) &&
      weapons.includes('Family = CustomDamageFamily Index = 5') &&
      weapons.includes('MaxSpeedGRU = 150') &&
      weapons.includes('ProjectileSpeedGRU = 300') &&
      categoryName('410', definitions) === undefined &&
      categoryName('MinMax_SAM', definitions) === 'SAM' &&
      genericTunings('AutocanonHE', false, true).length === 0 &&
      JSON.stringify(targetReads) === JSON.stringify([`${gfx}/Enums/WeaponsMinMaxCategory.ndf`]),
    {
      reason:
        'Custom ammunition rules changed protected weapons, missed inline objects, or multiplied a missile range twice.',
      suggestion:
        'Preserve canonical ammo, classify exact categories, and reuse the authored WTO edits.',
      details: [JSON.stringify(ranges), ...Object.values(results)],
    },
  );
  return {
    results: [
      {
        name: 'Custom ammunition preserves native balance and tunes hoisted and inline objects',
        status: 'passed' as const,
      },
    ],
  };
}
