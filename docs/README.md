# Your WTO setup

[← WTO showcase](../README.md) · [Install](#install) · [Customize](#customize-wto)

Choose the published [Steam Workshop mod](https://steamcommunity.com/sharedfiles/filedetails/?id=3387658237) to play, or use this guide for a local build.

## Install

You need Windows and WARNO. Git is not required.

1. In Steam: **WARNO → Properties → Installed Files → Browse**.
2. Open `Mods` and run `CreateNewMod.bat YourModName`.
3. Download [`Deploy-WTO.bat`](https://github.com/Yokaiste/WTO/releases/latest) into that new
   folder, beside `CommonData` and `GameData`, and double-click it.

It installs YMB, the YSM configuration package, and the WTO configuration package, then asks
whether to apply them. It never changes live WARNO files unless you answer yes.

## Customize WTO

Put local changes in `YMB/ymb.config.yaml`. The installer preserves this file when
updating YMB and the mod packages:

```yaml
version: 1
mods:
  wto:
    variables:
      tuning:
        ammunitionRangeMultiplier: 1.5
        missileRangeMultiplier: 1.5
        roadSpeedMultiplier: 1.1
        bmp2:
          salvoSize: 6
      branding:
        YSM: My tactical pack
        YSMTIT: My tactical pack
```

Only the named values change: the other `tuning` and `bmp2` settings stay at their
defaults. Shared ammunition tuning is in [config/ymb.mod.yaml](../config/ymb.mod.yaml);
unit, smoke, damage-state, capacity and branding defaults live in their owning patch configs.
The override example above still works across these scopes. Multipliers scale the
original value; values ending in `Gru` use game distance units. Preserve quoted NDF
float values such as `'2.0'` when editing them.

`branding` is a localisation-token-to-text map. The shipped YSM tokens are `YSM`
(display name), `YSMTIT` (title), and `YSMMT` (byline). Older WTO configs used
`displayName`, `menuTitle`, and `menuByline`; migrate those entries to the tokens above.
Branding leaves unrelated rows unchanged and does nothing if those tokens are absent.

For contributors, ammunition rules are ordinary YMB bulk operations in the feature's
`rules.json`, keyed by purpose. The base patch and compatibility adapter consume the
same definitions. Change a rule there; no sibling-YAML reader or copied balance table needs
updating. Compatibility's category mapping and YSM exclusions stay in its own feature.
The former `gfx` path alias and `vehicleStunDamageLevelsPack` representation are removed;
file paths and the native damage-pack reference are part of the authored operations.

From the WARNO mod folder, preview the combined selection:

```bat
YMB\YMB.bat build --mod ysm --mod wto
```

Inspect `YMB/.ymb-build/output`, then apply with the same selection and `sync --yes`.
Compile the WARNO mod using the game's generation step before loading it in game.
WTO builds after YSM, so its balance rules apply on top of the sandbox content.

You can disable individual features under `mods.wto.patches` using their ids, for example
`wto.ysm_branding: { enabled: false }`. See the [YMB configuration guide](../../../docs/configuration.md)
for shared overrides and dependency handling. To undo the combined installation:

```bat
YMB\YMB.bat recover --mod ysm --mod wto --yes
```

Keep `YMB/.ymb-state` for recovery. Back up local settings before a manual install.
