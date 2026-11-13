# Use and customize WTO

For a ready-made version, [subscribe on Steam](https://steamcommunity.com/sharedfiles/filedetails/?id=3387658237) and activate the mod in WARNO's Mod Center. Use a local build when you want to change its settings.

## Install a local copy

1. In Steam, open WARNO's installed files, then open its `Mods` folder. Open a terminal in `Mods` and run:

   ```powershell
   .\CreateNewMod.bat MyOverhaul
   ```

2. Download `Deploy-WTO.bat` from the [latest release](https://github.com/Yokaiste/WTO/releases/latest). Put it in `WARNO\Mods\MyOverhaul`, beside `GameData` and `CommonData`, then double-click it.
3. The installer downloads **YMB, YSM, and WTO**. Choose **N** at the apply prompt if you want WTO alone or want to edit settings first. **Y** applies YSM and WTO together.

The installer supplies the runtime and mod projects. You need Windows and an internet connection; Git and a separate Bun installation are unnecessary. WARNO's mod scripts also need Python; follow the setup in the game's `Mods/ReadMe.txt` if those scripts fail to start.

## Choose your build

Double-click `YMB/YMB.bat` in the new mod folder. Use one row consistently for both commands:

| Version      | Preview                     | Apply after review               |
| ------------ | --------------------------- | -------------------------------- |
| WTO alone    | `build --mod wto`           | `sync --mod wto --yes`           |
| YSM with WTO | `build --mod ysm --mod wto` | `sync --mod ysm --mod wto --yes` |

Read the preview in `YMB/.ymb-build/output`, then apply it. Finally, run `GenerateMod.bat` in `MyOverhaul` and activate the local mod in WARNO's Mod Center. YMB prepares the files; WARNO's script compiles them for play.

WTO applies after YSM when both are selected. If you already synced YSM and now want WTO alone, first run `recover --mod ysm --mod wto --yes`, then build and sync WTO alone. A narrower sync is not a request to uninstall another mod's tracked files.

## Change a setting

Edit `YMB/ymb.config.yaml`. Merge these values into its existing `mods` section and keep other settings:

```yaml
mods:
  wto:
    variables:
      multipliers:
        range: 0.9
      tuning:
        projectileSpeedMultiplier: 1.1
```

This reduces the ranges controlled by WTO's range scale by 10% and increases its projectile-speed setting by 10%. These multiply WTO's designed values, not vanilla values. A multiplier of `1.0` keeps WTO's default.

Shared scales in `multipliers` include `range`, `speed`, `vision`, `supply`, `timing`, `dispersion`, `suppression`, and `ammunition`. Each affects the rules that use it. Try one change at a time; doubling several related scales can make your test much more exciting than informative.

To disable one patch, use its `id` from `config/patch/.../ymb.patch.yaml`:

```yaml
mods:
  wto:
    patches:
      wto.ysm_branding:
        enabled: false
```

This keeps YSM's menu title when using both mods. Add it to your existing `mods.wto` mapping; do not paste a second `mods` section. Run the same preview, apply, and compile steps after editing.

## Update and undo

Run the installer again to update the downloaded projects. It keeps `YMB/ymb.config.yaml` and saved-deck `*.store.json` identities, but replaces other downloaded configuration files. Keep authored changes in your own checkout if you edit those files.

Keep `YMB/.ymb-state` for recovery. To restore YMB's saved game files, open `YMB.bat` and run:

```text
recover --mod wto --yes
```

For the combined version, use `recover --mod ysm --mod wto --yes`. Run `GenerateMod.bat` afterwards so the compiled mod matches the restored files.

Recovery restores whole tracked files, including changes from other mods sharing those files. Rebuild and sync the combination you want afterwards.

## Help and further reading

`doctor` checks folder setup. `explain --mod wto` explains skipped patches; add `--mod ysm` for a combined build. Bring questions, feedback, and bug reports to the [YSM Community](https://discord.gg/33Sqn6dTjf). For build problems, use its troubleshooting forum and include the command, error, YMB version, and selected mods.

Read [YMB's guide](https://github.com/Yokaiste/YMB/blob/master/docs/README.md) for your own patches and game-update steps. WTO uses the [MIT License](LICENSE); YSM and any other included projects keep their own terms.
