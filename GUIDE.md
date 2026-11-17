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
3. The installer downloads WTO and offers YSM too; choose **N** for WTO alone or **Y** for the combined build. It selects the newest YMB supported by every selected project. If none exists, it offers the latest YMB with a compatibility warning. Cancel unless you intend to test that unsupported combination.
4. Deployment syncs the selected projects and shows the next steps. Run `GenerateMod.bat`, then activate your local mod in WARNO's Mod Center. To change settings first, follow the sections below and compile after your own sync.

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

Read the preview in `YMB/.ymb/build/output`, then apply it. Finally, run `GenerateMod.bat` in `MyOverhaul` and activate the local mod in WARNO's Mod Center. YMB prepares the files; WARNO's script compiles them for play. Double-click `YMB.exe` when you prefer its interactive menu.

WTO works without YSM. Its YSM menu-branding patch makes no changes when YSM's dictionary is absent.

WTO applies after YSM when both are selected. If you already synced YSM and now want WTO alone, first run `recover --mod ysm --mod wto --yes`, then build and sync WTO alone. Sync stops if a narrower selection would erase an excluded contributor from a shared file.

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

Back up settings, `*.store.json` identities, other projects, and source edits outside YMB. Close running YMB commands and rerun the installer. It detects YMB and asks whether to update, warning that the whole folder will be removed. After checking the downloads, it restores every tracked live file, including manual edits and changes from other mods, replaces YMB, then syncs the new selection. Choosing WTO alone removes an installed YSM project and recovers its live changes. Restore compatible settings and identities from your backup when needed.

Use `--preserve-user-data` to keep compatible settings from both YMB config files and selected projects' `config/.ymb/` storage. Local overrides take precedence; settings for removed projects are dropped. The installer checks preserved data before replacement. This requires the default YMB folder layout; keep an external backup for source edits and other projects.

Git checkouts and builder source trees are rejected; recovery failure stops replacement. Keep `.ymb-install` after an interrupted update and rerun the installer. Successful deployment removes temporary files.

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
