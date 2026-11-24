# WTO agent rules

WTO owns its gameplay policy; YMB supplies the builder and public SDK.
Read [GUIDE.md](GUIDE.md), the affected feature, and Git status when available.

## Scope and preservation

- Preserve existing work. Do not commit, post messages, or publish without a request.
- Change YMB only when the task includes a tool change; follow its source rules.
  Use the public SDK for mod work. Do not copy its parser or import builder internals.
- Follow [LICENSE](LICENSE), [NOTICE](NOTICE), and all included material's terms.
  A rights holder may authorize exceptions only for their own work. Preserve attribution.
- Edit authored inputs. Preserve ids, GUIDs, localisation keys, generated owners,
  and persistent identity stores. Do not treat those stores as caches.
- Live writes, recovery, compilation, publishing, and identity resets need authorization
  from the task. Use previews for ordinary checks; do not repeat an approval already given.
- For a suspected YMB defect, offer a report for the [community troubleshooting forum](https://discord.gg/33Sqn6dTjf)
  with version, command, expected/actual output, diagnostic, and minimal reproduction.

## Implementation: Ponytail

Trace callers, settings, tests, and output. Remove needless work; reuse the feature's
helper, YMB API, standard library, or installed dependency before adding code. Keep
policy with its owner. Share a mechanism only when independent callers need it.
Finish changed callers and examples together. Preserve validation, errors, stable
identities, and recovery. Avoid repeated full-file scans. Test nontrivial logic with
small invented inputs; do not snapshot prose or repeat production tuning values.
Comment hidden constraints only; use `shortcut: <limit>, <when to upgrade>` for a deliberate limit.

## Balance and compatibility

- Ammunition, missiles, units, and capacities own their rules and checks. A global value must intentionally coordinate features; equal numbers alone do not justify sharing.
- Ammunition features own their named operations in `rules.json`. Base generators and compatibility use those same definitions. Keep external-mod categories and exclusions in compatibility.
- Compatibility adapts WTO rules to another mod. It must not create a second balance model, infer rule identity from expression spelling, or put WTO policy into YMB.
- YSM menu branding belongs to the branding patch. It must not change combat rules or make a WTO-only build depend on YSM.
- Objects merge; lists replace. Folder structure does not create variable inheritance. Keep supported overrides working through YMB's config rules.
- Keep designed factors and fixed overrides beside their operations. Useful user multipliers default to `1.0`; preserve exclusions and fixed limits for deliberately extreme values.
- Similar selectors can represent different weapon roles. Compare their purpose before merging them.
- Installer updates warn and ask before full recovery and replacement of the entire YMB folder. Back up settings, identities, and authored changes outside YMB first. Recovery restores whole tracked files, including files shared by several mods.

## Writing: Humanizer

Read affected text fully, cut filler and repeated claims, then check facts and examples.
Use plain verbs, sentence case, and no em or en dashes in prose. Keep commands, paths,
tokens, codes, product names, and measured claims exact. Keep instructions here concise
and neutral. User docs may use occasional humor; errors still need a cause and next action.
READMEs showcase the mod; guides teach tasks. Keep documentation media in docs/assets.
Use useful visuals and named navigation at both ends of guides. Showcase READMEs
have no previous/next tables; place links beside the relevant text.
Call editable inputs a mod project, output a preview, and the target a WARNO mod folder.
YMB sync applies files; WARNO's tools compile them.

## Checks and delivery

In a portable installation, use `YMB.exe build --mod wto --json` and
`YMB.exe build --mod ysm --mod wto --json` from the containing YMB folder.
Use `--json` for inspection commands too; keep the human progress display out of parsers.

From this mod project's root, check the native generator:

```powershell
cargo fmt --manifest-path config/generator/Cargo.toml --check
cargo clippy --manifest-path config/generator/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path config/generator/Cargo.toml --locked
```

Configuration and output changes need both previews with `--require-all`. In a source checkout, use `cargo run --release -p ymb -- build` with the same arguments. Build already validates. Deployment and release code belongs to the SDK's shared `distribution` module; the project supplies its name, version, repository, and optional YSM suggestion.
Keep persistent generated data and generator binaries in config/.ymb through the
SDK storage helpers. Session work belongs in YMB/.ymb/mods/wto. Keep dist beside
ymb.mod.yaml; ship config plus explicit release.include paths. Download URLs point
directly to HTTPS archives, including optional dependencies. Rust code uses src.

Verify fresh and updated standalone installs, optional YSM, checksums, compiler-free operation, full recovery before replacement, interrupted updates, sync, cleanup, and placement rejection in disposable fixtures. Inspect WTO's diff and any YSM identity changes after a combined preview. Report the result, checks, anything unverified, and remaining risk.
