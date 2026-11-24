mod aircraft;
mod ammunition;
mod branding;
mod rules;

use ymb_sdk::{GeneratorEntry, TestEntry};

fn main() {
    ymb_sdk::run(
        &[
            GeneratorEntry {
                name: "aircraft",
                generate: aircraft::generate,
            },
            GeneratorEntry {
                name: "ammunition",
                generate: |ctx| rules::generate(ctx, rules::AMMUNITION),
            },
            GeneratorEntry {
                name: "ammunition_missiles",
                generate: |ctx| rules::generate(ctx, rules::MISSILES),
            },
            GeneratorEntry {
                name: "missiles",
                generate: |ctx| rules::generate(ctx, rules::FLIGHT),
            },
            GeneratorEntry {
                name: "compatibility",
                generate: ammunition::generate,
            },
            GeneratorEntry {
                name: "branding",
                generate: branding::generate,
            },
        ],
        &[
            TestEntry {
                name: "compatibility",
                run: ammunition::checks,
            },
            TestEntry {
                name: "branding",
                run: branding::checks,
            },
        ],
    );
}
