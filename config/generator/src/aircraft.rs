use std::collections::{BTreeMap, HashMap};
use ymb_sdk::{Context, GeneratedFile, Result, ndf};

const UNITS: &str = "GameData/Generated/Gameplay/TacticalUnits.ndf";
const WEAPONS: &str = "GameData/Generated/Gameplay/Gfx/WeaponDescriptor.ndf";
const AMMO: [&str; 2] = [
    "GameData/Generated/Gameplay/Gfx/Ammunition.ndf",
    "GameData/Generated/Gameplay/Gfx/AmmunitionMissiles.ndf",
];

fn field<'a>(source: &'a str, name: &str) -> Result<Option<&'a str>> {
    Ok(ndf::read::direct_fields(source)?
        .into_iter()
        .find(|f| f.name == name)
        .map(|f| source[f.value].trim()))
}
fn number(source: &str, name: &str) -> Result<Option<f64>> {
    Ok(field(source, name)?
        .and_then(|raw| ndf::values::parse_scalar(raw).value.as_f64())
        .filter(|v| v.is_finite()))
}
fn reference(raw: &str) -> Option<&str> {
    let name = raw
        .strip_prefix("$/GFX/Weapon/")
        .or_else(|| raw.strip_prefix("~/"))
        .unwrap_or(raw);
    (!name.is_empty() && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'))
        .then_some(name)
}
fn clone_descriptor(source: &str, name: &str) -> Result<String> {
    let block = ndf::scan(source)?.into_iter().next().ok_or_else(|| {
        ymb_sdk::failure("Missing descriptor body.", "Check the aircraft weapon.")
    })?;
    let body = block.body.ok_or_else(|| {
        ymb_sdk::failure("Missing descriptor body.", "Check the aircraft weapon.")
    })?;
    ndf::patch::reidentify_ids(
        &format!("export {name} is {}{}\n", block.type_name, &source[body]),
        &format!("wto.compatibility.aircraft:{name}"),
    )
}

fn capped(source: &str, base: &str, limit: f64) -> Result<Option<String>> {
    if field(source, "TirIndirect")?.is_some_and(|v| ndf::values::parse_scalar(v).value == true)
        || field(source, "MissileDescriptor")?.is_some_and(|v| v != "nil")
    {
        return Ok(None);
    }
    let mut edits = Vec::new();
    for f in ndf::read::direct_fields(source)?
        .into_iter()
        .filter(|f| ["MaximumRangeGRU", "MaximumRangeHelicopterGRU"].contains(&f.name))
    {
        let current = ndf::values::parse_scalar(&source[f.value.clone()])
            .value
            .as_f64();
        if let (Some(current), Some(original)) = (current, number(base, f.name)?) {
            let cap = limit.floor().max(original);
            if current > original && current > cap {
                edits.push(ndf::patch::Edit {
                    range: f.value,
                    text: cap.to_string(),
                });
            }
        }
    }
    if edits.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ndf::patch::splice(source, edits)?))
    }
}

fn ammunition(files: &HashMap<String, String>) -> Result<HashMap<String, (&str, &str)>> {
    let mut result = HashMap::new();
    for path in AMMO.into_iter().chain([WEAPONS]) {
        for block in ndf::read::objects(&files[path])?
            .into_iter()
            .filter(|b| b.type_name == "TAmmunitionDescriptor")
        {
            if let Some(name) = block.name {
                result.insert(name.to_owned(), (path, &files[path][block.span]));
            }
        }
    }
    Ok(result)
}

fn adapt(
    current: &HashMap<String, String>,
    base: &HashMap<String, String>,
) -> Result<Vec<(String, String)>> {
    let ammo = ammunition(current)?;
    let base_ammo = ammunition(base)?;
    let managers: HashMap<_, _> = ndf::scan(&current[WEAPONS])?
        .into_iter()
        .filter(|b| b.type_name == "TWeaponManagerModuleDescriptor")
        .filter_map(|b| b.name.map(|name| (name, &current[WEAPONS][b.span])))
        .collect();
    let base_managers: HashMap<_, _> = ndf::scan(&base[WEAPONS])?
        .into_iter()
        .filter(|b| b.type_name == "TWeaponManagerModuleDescriptor")
        .filter_map(|b| b.name.map(|name| (name, &base[WEAPONS][b.span])))
        .collect();
    let mut additions: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut unit_edits = Vec::new();
    for unit in ndf::scan(&current[UNITS])?
        .into_iter()
        .filter(|b| b.type_name == "TEntityDescriptor")
    {
        let Some(unit_name) = unit.name else { continue };
        let unit_text = &current[UNITS][unit.span.clone()];
        let fields = ndf::read::fields(unit_text)?;
        let numeric = |name| {
            fields.iter().find(|f| f.name == name).and_then(|f| {
                ndf::values::parse_scalar(&unit_text[f.value.clone()])
                    .value
                    .as_f64()
            })
        };
        let (Some(altitude), Some(minimum)) = (numeric("AltitudeGRU"), numeric("AltitudeMinGRU"))
        else {
            continue;
        };
        if !altitude.is_finite() || !minimum.is_finite() || altitude <= minimum {
            continue;
        }
        // Leave a float32 margin at the engine's one-degree dive boundary.
        let limit = (altitude - minimum) / 1_f64.to_radians().tan() * 0.99999;
        let Some(modules) = fields.iter().find(|f| f.name == "ModulesDescriptors") else {
            continue;
        };
        for entry in ndf::read::entries(&unit_text[modules.value.clone()])? {
            let Some(manager_name) = reference(entry.text.trim()) else {
                continue;
            };
            let (Some(manager), Some(base_manager)) =
                (managers.get(manager_name), base_managers.get(manager_name))
            else {
                continue;
            };
            let mounts: Vec<_> = ndf::read::objects(manager)?
                .into_iter()
                .filter(|b| b.type_name == "TMountedWeaponDescriptor")
                .collect();
            let base_mounts: Vec<_> = ndf::read::objects(base_manager)?
                .into_iter()
                .filter(|b| b.type_name == "TMountedWeaponDescriptor")
                .collect();
            let mut edits = Vec::new();
            for (index, mount) in mounts.iter().enumerate() {
                let text = &manager[mount.span.clone()];
                let Some(ammunition_field) = ndf::read::direct_fields(text)?
                    .into_iter()
                    .find(|f| f.name == "Ammunition")
                else {
                    continue;
                };
                let value = text[ammunition_field.value.clone()].trim();
                let Some(base_mount) = base_mounts.get(index) else {
                    continue;
                };
                let Some(base_value) = field(&base_manager[base_mount.span.clone()], "Ammunition")?
                else {
                    continue;
                };
                let replacement = if let (Some(name), Some(base_name)) =
                    (reference(value), reference(base_value))
                {
                    let (Some((path, text)), Some((_, original))) =
                        (ammo.get(name), base_ammo.get(base_name))
                    else {
                        continue;
                    };
                    let Some(changed) = capped(text, original, limit)? else {
                        continue;
                    };
                    let clone = format!("WTO_Dive_{unit_name}_{name}");
                    additions
                        .entry((*path).into())
                        .or_default()
                        .insert(clone.clone(), clone_descriptor(&changed, &clone)?);
                    format!("$/GFX/Weapon/{clone}")
                } else if value.starts_with("TAmmunitionDescriptor")
                    && base_value.starts_with("TAmmunitionDescriptor")
                {
                    let Some(changed) = capped(value, base_value, limit)? else {
                        continue;
                    };
                    changed
                } else {
                    continue;
                };
                edits.push(ndf::patch::Edit {
                    range: mount.span.start + ammunition_field.value.start
                        ..mount.span.start + ammunition_field.value.end,
                    text: replacement,
                });
            }
            if edits.is_empty() {
                continue;
            }
            let clone = format!("WTO_Dive_{unit_name}_{manager_name}");
            additions.entry(WEAPONS.into()).or_default().insert(
                clone.clone(),
                clone_descriptor(&ndf::patch::splice(manager, edits)?, &clone)?,
            );
            unit_edits.push(ndf::patch::Edit {
                range: unit.span.start + modules.value.start + entry.span.start
                    ..unit.span.start + modules.value.start + entry.span.end,
                text: format!("$/GFX/Weapon/{clone}"),
            });
        }
    }
    let mut outputs = Vec::new();
    if !unit_edits.is_empty() {
        outputs.push((
            UNITS.into(),
            ndf::patch::splice(&current[UNITS], unit_edits)?,
        ));
    }
    for (path, blocks) in additions {
        outputs.push((
            path.clone(),
            format!(
                "{}\n{}",
                current[&path],
                blocks.into_values().collect::<Vec<_>>().join("\n")
            ),
        ));
    }
    Ok(outputs)
}

pub fn generate(ctx: &mut Context) -> Result<Vec<GeneratedFile>> {
    let paths = [UNITS, WEAPONS, AMMO[0], AMMO[1]];
    let current = ctx.read_targets(&paths)?;
    let base = ctx.read_base_targets(&paths)?;
    let outputs = adapt(&current, &base)?;
    if !outputs.is_empty() {
        ctx.notice("Limited WTO-increased aircraft gun ranges using private weapon copies.")?;
    }
    outputs
        .into_iter()
        .map(|(path, text)| ctx.output_text(&path, text))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inline_aircraft_ammunition_keeps_guided_and_indirect_behavior() {
        let base: HashMap<String, String> = HashMap::from([
            (UNITS.into(), "Plane is TEntityDescriptor(ModulesDescriptors = [TMovement(AltitudeGRU = 40 AltitudeMinGRU = 5), $/GFX/Weapon/Gun])".into()),
            (WEAPONS.into(), "Gun is TWeaponManagerModuleDescriptor(TurretDescriptorList = [TTurret(MountedWeaponDescriptorList = [TMountedWeaponDescriptor(Ammunition = TAmmunitionDescriptor(DescriptorId = GUID:{12345678-1234-1234-1234-123456789abc} MaximumRangeGRU = 1500))])])".into()),
            (AMMO[0].into(), "".into()), (AMMO[1].into(), "".into()),
        ]);
        let mut current = base.clone();
        current.insert(WEAPONS.into(), base[WEAPONS].replace("1500", "4000"));
        let output: HashMap<_, _> = adapt(&current, &base).unwrap().into_iter().collect();
        let clone = ndf::scan(&output[WEAPONS])
            .unwrap()
            .into_iter()
            .find(|b| b.name.is_some_and(|name| name.starts_with("WTO_Dive_")))
            .unwrap();
        let clone = &output[WEAPONS][clone.span];
        assert!(clone.contains("MaximumRangeGRU = 2005"));
        assert!(!clone.contains("GUID:{12345678-1234-1234-1234-123456789abc}"));
        for property in ["TirIndirect = True", "MissileDescriptor = ~/Guided"] {
            assert!(
                capped(
                    &format!("A is TAmmunitionDescriptor(MaximumRangeGRU = 4000 {property})"),
                    "A is TAmmunitionDescriptor(MaximumRangeGRU = 1500)",
                    2005.
                )
                .unwrap()
                .is_none()
            );
        }
    }
    #[test]
    fn low_aircraft_do_not_reduce_shared_ground_ammunition() {
        let mut base: HashMap<String, String> = HashMap::from([
            (UNITS.into(), "Plane is TEntityDescriptor(ModulesDescriptors = [TAirplaneMovementModuleDescriptor(AltitudeGRU = 40 AltitudeMinGRU = 5), $/GFX/Weapon/Gun])\nTank is TEntityDescriptor(ModulesDescriptors = [$/GFX/Weapon/Gun])".into()),
            (WEAPONS.into(), "Gun is TWeaponManagerModuleDescriptor(TurretDescriptorList = [TTurret(MountedWeaponDescriptorList = [TMountedWeaponDescriptor(Ammunition = $/GFX/Weapon/Round)])])".into()),
            (AMMO[0].into(), "Round is TAmmunitionDescriptor(DescriptorId = GUID:{12345678-1234-1234-1234-123456789abc} MaximumRangeGRU = 1500)".into()),
            (AMMO[1].into(), "".into()),
        ]);
        let mut current = base.clone();
        current
            .get_mut(AMMO[0])
            .unwrap()
            .replace_range(.., &base[AMMO[0]].replace("1500", "4000"));
        let outputs: HashMap<_, _> = adapt(&current, &base).unwrap().into_iter().collect();
        assert!(
            outputs[UNITS]
                .contains("Tank is TEntityDescriptor(ModulesDescriptors = [$/GFX/Weapon/Gun])")
        );
        assert!(outputs[AMMO[0]].contains("MaximumRangeGRU = 4000"));
        assert!(outputs[AMMO[0]].contains("MaximumRangeGRU = 2005"));
        for text in outputs.values() {
            ndf::validate(text).unwrap();
        }
        base = current;
        assert!(adapt(&base, &base).unwrap().is_empty());
    }
}
