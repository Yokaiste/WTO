<div align="center">

![WARNO Tactical Overhaul — range, movement, and battlefield tempo](publish/showcase.svg)

# WARNO Tactical Overhaul

### Make every approach matter.

[**Play on Steam**](https://steamcommunity.com/sharedfiles/filedetails/?id=3387658237) · [**Source package**](https://github.com/Yokaiste/WTO/releases/latest) · [**YSM Community**](https://discord.gg/VwsfZhuWQq) · [**Yuri’s WARNO Toolkit**](https://github.com/dary1337/yuri-warno-toolkit)

</div>

WTO reshapes engagement ranges, movement, vision, and weapon behavior across WARNO’s roster, changing where fights begin and how you approach them.

## Features

- **Longer reach.** Reworked ammunition and missile ranges change the space between opposing forces.
- **Movement and reconnaissance.** Revised road speeds and sight ranges reshape routes and positioning.
- **Weapon timing and handling.** Adjusted projectile speeds, missile flight, engagement cycles, artillery behavior, and selected weapon rules.
- **Customizable balance.** Tune the parts you want while keeping the remaining defaults.
- **YSM integration.** Apply WTO after YSM to bring its balance pass to the expanded sandbox, or build WTO alone against compatible WARNO mod data.
- **WTO × ANY MOD.** Bring the tactical overhaul to a mod you love with **[YWT](https://github.com/dary1337/yuri-warno-toolkit)**. Compatibility varies by mod.

## Work with the source

1. Create a WARNO mod with `Mods/CreateNewMod.bat YourModName`. Put [Deploy-WTO.bat](publish/deploy/Deploy-WTO.bat) in the new folder beside `GameData` and `CommonData`, then run it. The installer sets up [YMB](https://github.com/Yokaiste/YMB), YSM, and WTO, and asks before applying changes.
2. Keep local overrides in `YMB/ymb.config.yaml` under `mods.wto.variables` or `mods.wto.patches`. For source development, edit shared tuning in [config/ymb.mod.yaml](config/ymb.mod.yaml) and feature defaults or ammunition `rules.json` files in [config/patch](config/patch). Installer updates preserve local overrides but replace downloaded mod configs; keep authored changes in your own checkout.
3. From the WARNO mod folder, preview with `YMB\YMB.bat build --mod ysm --mod wto`. Inspect `YMB/.ymb-build/output`, apply with `YMB\YMB.bat sync --mod ysm --mod wto`, then compile using WARNO’s generation tools. Omit `--mod ysm` from both commands for WTO alone.

Keep `YMB/.ymb-state` for recovery. See YMB’s [configuration guide](../../docs/configuration.md) for overrides and feature selection, and [workflow guide](../../docs/workflow.md) for updates and recovery.

## Credits

- Owner and original WTO author: [ShadowofChernobyl](https://steamcommunity.com/id/ShadowofCherno/)
- Built with [YMB](https://github.com/Yokaiste/YMB)

## License

This source repository is available under the [MIT License](LICENSE).
WTO’s MIT license does not cover YSM or its code, which remain subject to [YSM’s own license](../YSM/LICENSE).
WARNO and its original game assets remain the property of Eugen Systems and are not relicensed by this repository.
