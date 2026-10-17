# WTO development

WTO owns a coordinated tactical balance model. In a YMB workspace, also follow the
builder and source-mod agent guides. The shared rationale is
[YMB's development principles](../../docs/development.md).

- Ammunition, missiles, units, and capacities own their balance rules and checks.
  Keep a value global only when it intentionally coordinates these features. Sharing
  a numeric value or putting everything under `tuning` does not establish shared ownership.
- Compatibility adapts those rules to other mod data. It must not maintain another
  balance model, infer rule identity from expression spelling, or introduce knowledge
  of WTO into YMB. Reuse a semantic rule through an explicit contract when needed.
  Ammunition features own named bulk operations in `rules.json`; their base scripts
  and compatibility import those same definitions. Keep category mapping and external
  mod exclusions in compatibility.
- YSM branding is an integration owned by the branding feature. It must not influence
  combat rules or make a standalone WTO build require YSM. Preserve the established
  localisation tokens when changing player text.
- Configuration exposes intended balance or presentation choices. NDF paths, serialized
  literals, parser details, and one-use aliases belong near their consumers. Do not
  create another config loader to make values appear more modular.
- Preserve independently useful rules when removing duplication. Similar-looking
  selectors can encode different weapon roles; compare their meaning before merging.
- Describe the tactical experience in player copy. Explain tuning in customization
  docs and implementation constraints beside the code that owns them.

Keep supported overrides working or document their migration with the change. Use
invented fixtures for tests, preserve existing work, and inspect WTO's own Git diff.
Leave legal files and live game data unchanged. Do not sync, recover, compile, publish,
stage, or commit unless the task authorizes that action.

For docs, run the builder's mod formatter and linter. For scripts, also run
`bun run check:mods`. For config or script changes, preview WTO alone and the layered
selection: `bun run ymb build --mod wto`, then
`bun run ymb build --mod ysm --mod wto`.
