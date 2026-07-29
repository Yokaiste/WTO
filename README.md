# WARNO Tactical Overhaul (WTO)

[Steam Workshop](https://steamcommunity.com/sharedfiles/filedetails/?id=3387658237) | [Source](https://github.com/Yokaiste/WTO) | [License](LICENSE)

WTO is a WARNO gameplay rebalance focused on longer ranges, slower pacing,
adjusted movement and vision, and weapon behavior changes across the roster.

This repository contains the source configuration used by YMB to build the mod.
It does not include WARNO game data; it describes patch operations applied to a
local WARNO installation.

## Install

You need Windows and WARNO. Git is not required.

1. In Steam: **WARNO → Properties → Installed Files → Browse**.
2. Open `Mods` and run `CreateNewMod.bat YourModName`.
3. Download [`Deploy-WTO.bat`](https://github.com/Yokaiste/WTO/releases/latest) into that new
   folder, beside `CommonData` and `GameData`, and double-click it.

It installs YMB, the YSM configuration package, and the WTO configuration package, then builds a
preview. It never changes live WARNO files on its own; it prints the sync command when it
finishes.

## Credits

- Owner and original WTO author: ShadowofChernobyl
- Built with [YMB](https://github.com/Yokaiste/YMB)

## Build

From `YMB/`, build the combined setup used by this mod:

```bat
bun run ymb build --mod ysm --mod wto
```

## License

This source repository is available under the [MIT License](LICENSE).
WARNO and its original game assets remain the property of Eugen Systems and are
not relicensed by this repository.
