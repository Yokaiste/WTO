# WTO development

WTO owns a coordinated tactical balance model. This guide governs WTO as an
independent product. YMB is its build dependency; changes to WTO do not authorize
changes to the builder or other mods.

- The repository README is a feature showcase, not a usage or configuration guide.
  WTO currently has no end-user documentation. Do not create it unless requested;
  keep development instructions here and implementation rationale beside its code.
- Author changes under `config/`. Use YMB's public `ymb/api` and declarative patches;
  never import builder internals or duplicate its NDF parser.
- Respect YMB's variable scope and merge contracts. Folders do not confer inheritance;
  objects merge and lists replace. Do not add a sibling-config loader.
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
- Keep designed factors and fixed overrides beside their operations. User-facing
  balance multipliers default to `1.0` and compose with category scales. Preserve
  fixed limits and exclusions for deliberately extreme values.
- Preserve independently useful rules when removing duplication. Similar-looking
  selectors can encode different weapon roles; compare their meaning before merging.
- Describe the tactical experience in player copy and preserve WTO's feature brands.
  Keep tuning rationale and implementation constraints beside the code that owns them.
- Keep installer updates compatible with local `ymb.config.yaml` overrides and
  authored identity stores. Verify this product's deployment changes in its own checkout.

Keep supported overrides working or report their migration with the change. Use
invented fixtures for tests, preserve existing work, and inspect WTO's own Git diff.
Leave legal files and live game data unchanged. Do not sync, recover, compile, publish,
stage, or commit unless the task authorizes that action.

For README or agent-guide changes, run YMB's `fix:mods` and `lint:mods` tasks.
For scripts, also run `bun run check:mods`. For config or script changes, preview WTO
alone and the layered
selection: `bun run ymb build --mod wto`, then
`bun run ymb build --mod ysm --mod wto`.
