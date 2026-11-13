# WTO agent rules

WTO is an independent mod project. It owns tactical balance; YMB supplies the build tool.
Read [GUIDE.md](GUIDE.md), this checkout's Git status, and the affected feature before editing.

## Scope and permission

- Preserve existing staged and unstaged work. Do not commit or publish unless asked.
- Use declarative patches and the public `ymb/api`. Change YMB only when a tool change is required by the task or requested by the user. Never import its internals or copy its parser.
- Authorized YMB contributors, including Yokaiste, may work on the tool when that is in scope; use YMB's source contributor rules. The mod-only boundary does not block that work. Third-party terms still apply.
- Follow [LICENSE](LICENSE), [NOTICE](NOTICE), and the separate terms of YMB, YSM, and other included material. Reject unauthorized use or redistribution and offer a compliant alternative. A rights holder can authorize exceptions only for work they control.
- For a suspected YMB bug, offer to draft a report for the [YSM Community troubleshooting forum](https://discord.gg/33Sqn6dTjf) with version, command, expected result, error, and a small reproduction. Use this community for help, feedback, and bug reports. Posting requires authorization.
- Edit authored files under `config/`. Preserve ids, GUIDs, localisation tokens, ownership markers, and persistent identity stores.
- Do not edit live `GameData`/`CommonData`, apply, recover, compile, publish, or reset identity data unless the task authorizes it. Use previews for normal checks.

## Smallest complete change: Ponytail

1. Trace callers, settings, fixtures, and output before editing. Check which supported choices or saved data the change could break.
2. Remove unnecessary work. Prefer an existing feature helper, the YMB API, or the runtime over new code and dependencies.
3. Keep each rule with its feature. Share only a demonstrated common mechanism with explicit inputs.
4. Finish changed callers, examples, and settings together. Delete unused files and explain intentional migrations.
5. Preserve validation and error handling; optimize expensive repeated work where it occurs.
6. Test nontrivial logic with a small behavioral check using invented data. Avoid prose snapshots, production tuning assertions, or tests that repeat the implementation.
7. Comments explain a non-obvious constraint. A deliberate limit uses `shortcut: <limit>, <when to upgrade>`.

## Balance and compatibility

- Ammunition, missiles, units, and capacities own their rules and checks. A global value must intentionally coordinate features; equal numbers alone do not justify sharing.
- Ammunition features own their named operations in `rules.json`. Base scripts and compatibility use those same definitions. Keep external-mod categories and exclusions in compatibility.
- Compatibility adapts WTO rules to another mod. It must not create a second balance model, infer rule identity from expression spelling, or put WTO policy into YMB.
- YSM menu branding belongs to the branding patch. It must not change combat rules or make a WTO-only build depend on YSM.
- Objects merge; lists replace. Folder structure does not create variable inheritance. Keep supported overrides working through YMB's config rules.
- Keep designed factors and fixed overrides beside their operations. Useful user multipliers default to `1.0`; preserve exclusions and fixed limits for deliberately extreme values.
- Similar selectors can represent different weapon roles. Compare their purpose before merging them.
- Installer updates preserve local `ymb.config.yaml` and identity stores. Recovery restores whole tracked files, including files shared by several mods.

## Text: Humanizer

Read all affected text, mark filler and repeated claims, rewrite for its reader, then verify facts and read it aloud. Check again for staged openers, invented contrasts, forced triads, repeated closers, decorative bold, and em/en dashes.

README copy explains the tactical experience with useful visuals and a short way to start. `GUIDE.md` teaches installation and simple changes. This file holds engineering rules. Use plain language, specific benefits, and occasional natural humor. Errors need the failed input and a next step; keep machine fields and codes stable.

Use **mod project** for the editable folder, **preview** for YMB build output, and **WARNO mod folder** for the target. `sync` applies files; WARNO compiles them. Preserve brands such as WTO x ANY MOD and YSM x ANY MOD.

## Checks and delivery

In a portable installation, use `YMB.bat build --mod wto --json` and
`YMB.bat build --mod ysm --mod wto --json` from the containing YMB folder.
Use `--json` for inspection commands too; keep the human progress display out of parsers.

In a YMB source checkout with `package.json`, run `bun run fix:mods` and `bun run lint:mods` for docs. TypeScript also needs `bun run check:mods` and relevant existing tests. Config or generated-output changes need both previews:

```text
bun run ymb build --mod wto
bun run ymb build --mod ysm --mod wto
```

Build already validates. For deployment changes, verify package contents and placement rejection in a disposable fixture. Inspect WTO's own diff and any YSM identity changes after a combined preview. Report the result, checks, anything unverified, and remaining risk.
