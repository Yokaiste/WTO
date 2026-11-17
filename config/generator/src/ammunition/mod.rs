mod fixtures;
mod patch;

use crate::rules::{self, AMMUNITION, FLIGHT, GFX, MISSILES, Rule};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use ymb_sdk::{Context, GeneratedFile, Result, TestResult, ndf};

const FILES: [&str; 4] = [
    AMMUNITION.file,
    MISSILES.file,
    "WeaponDescriptor.ndf",
    FLIGHT.file,
];

pub fn category_name(raw: Option<&str>, definitions: &Map<String, Value>) -> Option<String> {
    let raw = raw?.trim();
    let symbolic = raw.rsplit('/').next()?;
    if symbolic.starts_with("MinMax_") && definitions.contains_key(symbolic) {
        return Some(symbolic[7..].into());
    }
    let number = raw
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())?;
    let mut names = definitions
        .iter()
        .filter(|(name, value)| name.starts_with("MinMax_") && value.as_f64() == Some(number));
    let name = names.next()?.0;
    names.next().is_none().then(|| name[7..].to_owned())
}

pub fn generic_tunings(category: Option<&str>, missile: bool, indirect: bool) -> Vec<&'static str> {
    if missile {
        return if category == Some("SAM") {
            vec!["range", "aaCycle"]
        } else {
            vec!["range"]
        };
    }
    let mut result = Vec::new();
    if !indirect
        && matches!(
            category,
            Some(
                "AutocanonHE"
                    | "CanonAP"
                    | "DCA"
                    | "MMG_HMG"
                    | "LAW"
                    | "Assault_rifle"
                    | "SMG"
                    | "Gatling"
                    | "Grenade"
                    | "inf_MMG"
                    | "Roquette_AS"
                    | "inf_sniper"
            )
        )
    {
        result.push("range");
    }
    if !indirect
        && matches!(
            category,
            Some(
                "AutocanonHE" | "CanonAP" | "DCA" | "MMG_HMG" | "LAW" | "Assault_rifle" | "inf_MMG"
            )
        )
    {
        result.push("projectileSpeed");
    }
    if category == Some("obusier") {
        result.push("spread");
    }
    if matches!(category, Some("obusier" | "Mortier" | "MLRS")) {
        result.push("suppression");
    }
    if matches!(category, Some("DCA" | "Gatling")) {
        result.push("aaCycle");
    }
    result
}

fn field<'a>(text: &'a str, fields: &[ndf::read::Field<'_>], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| text[field.value.clone()].trim())
}

fn protected(objects: &[ndf::Block<'_>]) -> Vec<std::ops::Range<usize>> {
    objects
        .iter()
        .filter(|block| {
            block.name.is_some_and(|name| {
                let tail = name
                    .strip_prefix("WeaponDescriptor_Y")
                    .or_else(|| name.strip_prefix("Ammo_Y"));
                tail.is_some_and(|tail| {
                    ["S_", "C_", "N_", "Z_"]
                        .iter()
                        .any(|prefix| tail.starts_with(prefix))
                })
            })
        })
        .map(|block| block.span.clone())
        .collect()
}

fn in_protected(block: &ndf::Block<'_>, protected: &[std::ops::Range<usize>]) -> bool {
    protected
        .iter()
        .any(|root| root.start <= block.span.start && root.end >= block.span.end)
}

pub(super) fn extend(
    inputs: &HashMap<String, String>,
    rules: &[Rule],
    variables: &Value,
    identity: &ndf::operation::Identity,
) -> Result<Vec<(String, String)>> {
    let definitions =
        ndf::read::numeric_definitions(&inputs[&format!("{GFX}/Enums/WeaponsMinMaxCategory.ndf")])?;
    let mut categories = HashMap::new();
    let mut category = |raw: Option<&str>| -> Option<String> {
        let raw = raw?;
        categories
            .entry(raw.to_owned())
            .or_insert_with(|| category_name(Some(raw), &definitions))
            .clone()
    };
    let named: Vec<_> = rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| {
            rule.source != FLIGHT
                && rule.source != rules::CUSTOM
                && rule
                    .matcher
                    .conditions
                    .iter()
                    .any(|condition| condition.on == "name")
        })
        .collect();
    let mut contents = HashMap::new();
    let mut flight_names = HashSet::new();
    for file in FILES {
        let path = format!("{GFX}/{file}");
        let content = &inputs[&path];
        let objects = ndf::read::objects(content)?;
        let protected = protected(&objects);
        let top_level: HashSet<_> = ndf::scan(content)?
            .into_iter()
            .map(|block| block.span.start)
            .collect();
        let mut selected = Vec::new();
        for block in objects.iter().filter(|block| {
            block.type_name == "TAmmunitionDescriptor" && !in_protected(block, &protected)
        }) {
            let text = &content[block.span.clone()];
            let fields = ndf::read::direct_fields(text)?;
            let category = category(field(text, &fields, "MinMaxCategory"));
            let missile = field(text, &fields, "MissileDescriptor").filter(|value| *value != "nil");
            if matches!(category.as_deref(), Some("ATGM" | "AGM"))
                && let Some(missile) =
                    missile.filter(|value| value.starts_with("$/") || value.starts_with("~/"))
            {
                flight_names.insert(missile.rsplit('/').next().unwrap().to_owned());
            }
            let mut custom = Vec::new();
            for (index, rule) in rules
                .iter()
                .enumerate()
                .filter(|(_, rule)| rule.source == rules::CUSTOM)
            {
                if ndf::patch::matches(text, block.name, block.type_name, &rule.matcher)? {
                    custom.push(index);
                }
            }
            if !custom.is_empty() {
                selected.push((block, custom));
                continue;
            }
            // compatibility: vanilla Ammo_ roles retain their feature-owned exceptions.
            if block.name.is_some_and(|name| name.starts_with("Ammo_"))
                && [AMMUNITION.file, MISSILES.file].contains(&file)
            {
                continue;
            }
            let mut authored = Vec::new();
            for (index, rule) in &named {
                if ndf::patch::matches(text, block.name, block.type_name, &rule.matcher)? {
                    authored.push(*index);
                }
            }
            let mut indices = Vec::new();
            if !authored.is_empty() {
                indices.extend(
                    authored
                        .iter()
                        .copied()
                        .filter(|index| rules[*index].source.file != file || block.name.is_none()),
                );
            } else {
                let indirect = field(text, &fields, "TirIndirect")
                    .is_some_and(|raw| ndf::values::parse_scalar(raw).value == true);
                for tuning in generic_tunings(category.as_deref(), missile.is_some(), indirect) {
                    let source = if missile.is_some() {
                        MISSILES
                    } else {
                        AMMUNITION
                    };
                    indices.push(patch::rule_index(rules, source, tuning)?);
                }
            }
            if !authored.is_empty() && missile.is_some() && file != MISSILES.file {
                indices.push(patch::rule_index(rules, MISSILES, "range")?);
            }
            let mut pending = Vec::new();
            for index in indices {
                let rule = &rules[index];
                let applied = rule.source.file == file
                    && top_level.contains(&block.span.start)
                    && ndf::patch::matches(text, block.name, block.type_name, &rule.matcher)?;
                if !applied {
                    pending.push(index);
                }
            }
            if !pending.is_empty() {
                selected.push((block, pending));
            }
        }
        if !selected.is_empty() {
            let changed = patch::objects(content, &path, &selected, rules, variables, identity)?;
            if changed != *content {
                contents.insert(path, changed);
            }
        }
    }
    let mut outputs = Vec::new();
    let indices: Vec<_> = rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| rule.source == FLIGHT)
        .map(|(index, _)| index)
        .collect();
    for file in FILES {
        let path = format!("{GFX}/{file}");
        let content = contents.remove(&path);
        let text = content.as_deref().unwrap_or(&inputs[&path]);
        let objects = ndf::read::objects(text)?;
        let protected = protected(&objects);
        let mut parents = Vec::new();
        for block in objects
            .iter()
            .filter(|block| block.type_name == "TAmmunitionDescriptor")
        {
            let text = &text[block.span.clone()];
            let fields = ndf::read::direct_fields(text)?;
            if matches!(
                category(field(text, &fields, "MinMaxCategory")).as_deref(),
                Some("ATGM" | "AGM")
            ) {
                parents.push(block.span.clone());
            }
        }
        let selected: Vec<_> = objects
            .iter()
            .filter(|block| {
                block.type_name == "TEntityDescriptor"
                    && (file != FLIGHT.file
                        || !block
                            .name
                            .is_some_and(|name| name.starts_with("Descriptor_Missile_")))
                    && !in_protected(block, &protected)
                    && (block.name.is_some_and(|name| flight_names.contains(name))
                        || block.name.is_none()
                            && parents.iter().any(|parent| {
                                parent.start < block.span.start && parent.end >= block.span.end
                            }))
            })
            .map(|block| (block, indices.clone()))
            .collect();
        let result = if selected.is_empty() {
            content
        } else {
            Some(patch::objects(
                text, &path, &selected, rules, variables, identity,
            )?)
        };
        if let Some(result) = result.filter(|result| *result != inputs[&path]) {
            outputs.push((path, result));
        }
    }
    Ok(outputs)
}

pub fn generate(ctx: &mut Context) -> Result<Vec<GeneratedFile>> {
    let mut rules = Vec::new();
    for source in [AMMUNITION, MISSILES, FLIGHT, rules::CUSTOM] {
        rules.extend(rules::load(ctx, source)?);
    }
    let paths: Vec<_> = FILES
        .into_iter()
        .chain(["Enums/WeaponsMinMaxCategory.ndf"])
        .map(|file| format!("{GFX}/{file}"))
        .collect();
    let inputs = ctx.read_targets(&paths.iter().map(String::as_str).collect::<Vec<_>>())?;
    let identity = ctx.patch_identity("");
    extend(&inputs, &rules, &ctx.variables, &identity)?
        .into_iter()
        .map(|(file, text)| ctx.output_text(&file, text))
        .collect()
}

pub fn checks(_: &mut Context) -> Result<Vec<TestResult>> {
    fixtures::check()?;
    Ok(vec![TestResult::passed(
        "Custom ammunition keeps native balance and protected weapons",
    )])
}
