import type { BuildScript } from 'ymb/api';
import rules from './rules.json';

const apply: BuildScript = async (context) => {
  const file = 'GameData/Generated/Gameplay/Gfx/MissileDescriptors.ndf';
  return [
    {
      targetRelativePath: file,
      content: await context.tools.patch(await context.readTarget(file), {
        file,
        operations: Object.values(rules),
      }),
    },
  ];
};
export default apply;
