use super::*;
use serde_json::json;
use ymb_sdk::ensure;

pub(super) fn check() -> Result<()> {
    let rule = |field: &str, variable: &str| json!({"op":"bulk","match":{"conditions":[{"on":"type","is":"equals","value":"TAmmunitionDescriptor"}]},"edits":[{"field":field,"multiply":format!("${{tuning.{variable}}}")}]});
    let mut rules = rules::parse(AMMUNITION, &json!({"range":rule("MaximumRangeGRU","ammunitionRangeMultiplier"),"projectileSpeed":rule("ProjectileSpeedGRU","projectileSpeedMultiplier")}).to_string())?;
    let mut aa_cycle = rule("AimingTime", "aaLockTimeMultiplier");
    aa_cycle["match"] = json!({"conditions":[{"on":"name","is":"startsWith","value":"Ammo_SAM"}]});
    rules.extend(rules::parse(
        MISSILES,
        &json!({"range":rule("MaximumRangeGRU","missileRangeMultiplier"),"aaCycle":aa_cycle})
            .to_string(),
    )?);
    rules.extend(rules::parse(
        FLIGHT,
        &json!({"flight":rule("MaxSpeedGRU","missileFlightMultiplier")}).to_string(),
    )?);
    let variables = json!({"tuning":{"ammunitionRangeMultiplier":2,"projectileSpeedMultiplier":1.5,"missileRangeMultiplier":2,"aaLockTimeMultiplier":3,"missileFlightMultiplier":1.5}});
    let inputs = HashMap::from([
        (format!("{GFX}/Enums/WeaponsMinMaxCategory.ndf"), "MinMax_AutocanonHE is 41\nMinMax_SAM is 52\nMinMax_ATGM is 63".into()),
        (format!("{GFX}/Ammunition.ndf"), "Ammo_Custom is TAmmunitionDescriptor(MinMaxCategory = 41 MaximumRangeGRU = 100)\nAmmo_Protected is TAmmunitionDescriptor(MinMaxCategory = 63 MissileDescriptor = ~/Descriptor_Missile_Other)".into()),
        (format!("{GFX}/AmmunitionMissiles.ndf"), "Decooked_SAM is TAmmunitionDescriptor(MinMaxCategory = 52 MaximumRangeGRU = 200 AimingTime = 2 TimeBetweenTwoShots = 3 MissileDescriptor = ~/Missile_SAM)".into()),
        (format!("{GFX}/MissileDescriptors.ndf"), "CustomFlight is TEntityDescriptor(ModulesDescriptors = [TMovement(MaxSpeedGRU = 100 MaxAccelerationGRU = 20)])\nDescriptor_Missile_Other is TEntityDescriptor(MaxSpeedGRU = 120)".into()),
        (format!("{GFX}/WeaponDescriptor.ndf"), r#"Weapon is TWeaponManagerModuleDescriptor(Ammunition = [
            TAmmunitionDescriptor(MinMaxCategory = 41 // MissileDescriptor is only a comment
                MaximumRangeGRU = 100 ProjectileSpeedGRU = 200 Arme = TDamageTypeRTTI(Family = CustomDamageFamily Index = 5)),
            TAmmunitionDescriptor(MinMaxCategory = 410 MaximumRangeGRU = 100),
            TAmmunitionDescriptor(MinMaxCategory = MinMax_ATGM MaximumRangeGRU = 300 MissileDescriptor = ~/CustomFlight),
            TAmmunitionDescriptor(MinMaxCategory = 63 MaximumRangeGRU = 300 MissileDescriptor = TEntityDescriptor(ModulesDescriptors = [TMovement(MaxSpeedGRU = 100 MaxAccelerationGRU = 20)])),
            TAmmunitionDescriptor(MinMaxCategory = 41 MaximumRangeGRU = 100 TirIndirect = True),
            TAmmunitionDescriptor(MinMaxCategory = 41 MaximumRangeGRU = 100 TirIndirect = true),
        ])
        WeaponDescriptor_YC_Illegal is TWeaponManagerModuleDescriptor(Ammunition = TAmmunitionDescriptor(MinMaxCategory = 41 MaximumRangeGRU = 999))
        Decooked_CustomGun is TAmmunitionDescriptor(MinMaxCategory = 41 MaximumRangeGRU = 100 ProjectileSpeedGRU = 200)"#.into()),
    ]);
    let identity = ndf::operation::Identity {
        mod_id: "wto".into(),
        patch_id: "compatibility".into(),
        ..Default::default()
    };
    let results: HashMap<_, _> = extend(&inputs, &rules, &variables, &identity)?
        .into_iter()
        .collect();
    let weapons = &results[&format!("{GFX}/WeaponDescriptor.ndf")];
    let ranges = ndf::read::objects(weapons)?
        .into_iter()
        .filter(|block| block.type_name == "TAmmunitionDescriptor")
        .map(|block| {
            let text = &weapons[block.span];
            Ok(
                field(text, &ndf::read::direct_fields(text)?, "MaximumRangeGRU")
                    .unwrap()
                    .to_owned(),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let definitions =
        ndf::read::numeric_definitions(&inputs[&format!("{GFX}/Enums/WeaponsMinMaxCategory.ndf")])?;
    ensure(
        ranges == ["200", "100", "600", "600", "100", "100", "999", "200"]
            && !results.contains_key(&format!("{GFX}/Ammunition.ndf"))
            && results[&format!("{GFX}/AmmunitionMissiles.ndf")].contains("MaximumRangeGRU = 200")
            && results[&format!("{GFX}/AmmunitionMissiles.ndf")].contains("AimingTime = 6")
            && results[&format!("{GFX}/MissileDescriptors.ndf")].contains("MaxSpeedGRU = 150")
            && results[&format!("{GFX}/MissileDescriptors.ndf")]
                .contains("Descriptor_Missile_Other is TEntityDescriptor(MaxSpeedGRU = 120)")
            && weapons.contains("Family = CustomDamageFamily Index = 5")
            && weapons.contains("MaxSpeedGRU = 150")
            && weapons.contains("ProjectileSpeedGRU = 300")
            && category_name(Some("410"), &definitions).is_none()
            && category_name(Some("~/MinMax_SAM"), &definitions).as_deref() == Some("SAM")
            && generic_tunings(Some("AutocanonHE"), false, true).is_empty(),
        format!("Custom ammunition selection or scaling failed: {ranges:?}"),
        "Keep canonical ammunition, exact categories, indirect-fire exclusions, and protected YSM weapons intact.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compatibility_never_retunes_native_named_ammunition() {
        let rule = |field| json!({"op":"bulk","match":{"conditions":[{"on":"name","is":"startsWith","value":"Ammo_Original"}]},"edits":[{"field":field,"multiply":2}]});
        let rules = rules::parse(
            AMMUNITION,
            &json!({"range":rule("MaximumRangeGRU"),"projectileSpeed":rule("ProjectileSpeedGRU")})
                .to_string(),
        )
        .unwrap();
        let inputs = FILES
            .into_iter()
            .map(|file| (format!("{GFX}/{file}"), String::new()))
            .chain([(
                format!("{GFX}/Enums/WeaponsMinMaxCategory.ndf"),
                "MinMax_AutocanonHE is 41".into(),
            )])
            .collect::<HashMap<_, _>>();
        let mut inputs = inputs;
        inputs.insert(format!("{GFX}/{}", AMMUNITION.file), "Ammo_Renamed is TAmmunitionDescriptor(MinMaxCategory = 41 MaximumRangeGRU = 100 ProjectileSpeedGRU = 40)".into());
        let identity = ndf::operation::Identity::default();
        assert!(
            extend(&inputs, &rules, &json!({}), &identity)
                .unwrap()
                .is_empty()
        );
        assert!(
            extend(
                &inputs,
                &rules,
                &json!({"compatibility":{"allowMissingVanillaTargets":true}}),
                &identity,
            )
            .unwrap()
            .is_empty()
        );
    }
    #[test]
    fn custom_ammunition_covers_inline_hoisted_protected_and_missile_cases() {
        check().unwrap();
    }
    #[test]
    fn numeric_categories_reject_ambiguous_aliases() {
        let definitions =
            serde_json::from_value(json!({"MinMax_AutocanonHE":41,"MinMax_SAM":41})).unwrap();
        assert_eq!(category_name(Some("41"), &definitions), None);
        assert_eq!(
            category_name(Some("MinMax_SAM"), &definitions).as_deref(),
            Some("SAM")
        );
    }
}
