# Use and customize WTO

| Previous | You are here | Next |
| --- | --- | --- |
| [WTO overview](README.md) | Install and customize WTO | [YMB documentation](https://github.com/Yokaiste/YMB/blob/master/docs/README.md) |

For a ready-made version, [subscribe on Steam](https://steamcommunity.com/sharedfiles/filedetails/?id=3387658237) and activate the mod in WARNO's Mod Center. Use a local build when you want to change its settings.

## Install a local copy

1. In Steam, open WARNO's installed files, then open its `Mods` folder. Open a terminal in `Mods` and run:

   ```powershell
   .\CreateNewMod.bat MyOverhaul
   ```

2. Download `Deploy-WTO.exe` from the [latest release](https://github.com/Yokaiste/WTO/releases/latest). Put it in `WARNO\Mods\MyOverhaul`, beside `GameData` and `CommonData`, then double-click it.
3. Choose **N** for WTO alone or **Y** to add YSM. The installer looks for a stable YMB release that supports your selection, then checks previews. If it offers an unsupported version, cancel unless you intend to test it.
4. After the installer applies the selected mods, run `GenerateMod.bat` in `MyOverhaul` and activate your local mod in WARNO's Mod Center. To change settings first, follow the sections below and compile after syncing.

You need 64-bit Windows 10 or later and an internet connection for the installer. It supplies the portable builder and precompiled generator. Normal use needs no Git, compiler, or separate runtime. WARNO's mod scripts also need Python; follow the setup in the game's `Mods/ReadMe.txt` if those scripts fail to start.

Development archives are available from the [preview release](https://github.com/Yokaiste/WTO/releases/tag/preview). A deploy installer follows the direct archive URLs embedded in its mod configuration and dependency list.

```mermaid
flowchart LR
    A["Edit local settings"] --> B["Build + inspect"]
    B --> C["Sync"]
    C --> D["GenerateMod.bat"]
    D --> E["Enable and play"]
```

## Choose your build

Open PowerShell inside `YMB` in the new mod folder. Use one row consistently for both commands:

| Version      | Preview                     | Apply after review               |
| ------------ | --------------------------- | -------------------------------- |
| WTO alone    | `.\YMB.exe build --mod wto`           | `.\YMB.exe sync --mod wto --yes`           |
| YSM with WTO | `.\YMB.exe build --mod ysm --mod wto` | `.\YMB.exe sync --mod ysm --mod wto --yes` |

Review `YMB/.ymb/build/output`, then run the matching `sync` command to rebuild and apply that selection. Run `GenerateMod.bat` in `MyOverhaul` to compile it and activate the local mod in WARNO's Mod Center. Double-click `YMB.exe` to use the menu.

WTO works without YSM. Its YSM menu-branding patch makes no changes when YSM's dictionary is absent.

WTO applies after YSM when both are selected. To switch from YSM alone to WTO, first run `.\YMB.exe recover --mod ysm --yes`. If you applied both projects, run `.\YMB.exe recover --mod ysm --mod wto --yes`. Then build and sync WTO alone.

## Change a setting

Create or edit `YMB/.ymb/config.yaml` for this installation's settings. The commented root `YMB/ymb.config.yaml` holds shared defaults and stays with the repository. Merge these values into the local file's existing `mods` section, keeping other settings:

```yaml
version: 2
mods:
  wto:
    variables:
      multipliers:
        range: 0.9
      tuning:
        projectileSpeedMultiplier: 1.1
```

This reduces the ranges controlled by WTO's range scale by 10% and increases its projectile-speed setting by 10%. These multiply WTO's designed values, not vanilla values. A multiplier of `1.0` keeps WTO's default.

The `multipliers` section also controls `speed`, `vision`, `supply`, `timing`, `dispersion`, `suppression`, and `ammunition`. Change one value at a time so you can see its effect in a match.

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

Back up settings, `*.store.json` identities, other projects, and source edits outside YMB. Close running YMB commands and rerun the installer. After you confirm, it restores every tracked game file from saved originals, then checks that the new projects build. It replaces YMB and syncs your selection once the check passes. Choosing WTO alone also removes the installed YSM project.

Use `--preserve-user-data` to keep compatible settings from both YMB config files and selected projects' `config/.ymb/` storage. Local overrides take precedence; settings for removed projects are dropped. The installer checks preserved data before replacement. This requires the default YMB folder layout; keep an external backup for source edits and other projects.

Use Git to update a source checkout. If the installer is interrupted, leave `.ymb-install` in place and rerun it. If the build check fails after recovery, sync from the previous YMB folder to reapply your mods.

Keep `YMB/.ymb/state` for recovery. To restore YMB's saved game files, run this in PowerShell inside `YMB`:

```text
.\YMB.exe recover --mod wto --yes
```

For the combined version, use `recover --mod ysm --mod wto --yes`. Run `GenerateMod.bat` afterwards so the compiled mod matches the restored files.

Recovery restores whole tracked files, including changes from other mods sharing those files. Rebuild and sync the combination you want afterwards.

Changing YAML settings needs no Rust toolchain. WARNO compilation still follows sync.

## Help and further reading

`doctor` checks folder setup. `explain --mod wto` explains skipped patches; add `--mod ysm` for a combined build. Bring questions, feedback, and bug reports to the [YSM Community](https://discord.gg/33Sqn6dTjf). For build problems, use its troubleshooting forum and include the command, error, YMB version, and selected mods.

Read [YMB's guide](https://github.com/Yokaiste/YMB/blob/master/docs/README.md) for your own patches and game-update steps. WTO uses the [MIT License](LICENSE); YSM and any other included projects keep their own terms.

## Compatibility

External ammunition and active-protection adjustments live under `config/patch/compatibility/`. Aircraft range safety creates private weapon copies only when WTO increases a direct-fire range beyond the aircraft's dive limit. Shared ground weapons retain their values. Guided and indirect ammunition retain their own attack behavior.

For an overhaul that removes native roles, set `mods.wto.variables.compatibility.allowMissingVanillaTargets: true`. This also enables category-based fallback for renamed ammunition that keeps an Ammo_ prefix. Missing roles are reported; malformed data and ownership conflicts still fail. Leave this off for vanilla release checks. Vehicle-smoke tuning shared with active protection is `mods.wto.variables.tuning.vehicleSmokeSalvosMultiplier`.

| Previous | You are here | Next |
| --- | --- | --- |
| [WTO overview](README.md) | Install and customize WTO | [YMB documentation](https://github.com/Yokaiste/YMB/blob/master/docs/README.md) |
