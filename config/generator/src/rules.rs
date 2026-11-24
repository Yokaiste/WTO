use serde_json::Value;
use ymb_sdk::{Context, GeneratedFile, Result, ensure, failure, ndf};

pub const GFX: &str = "GameData/Generated/Gameplay/Gfx";

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Source {
    pub folder: &'static str,
    pub file: &'static str,
}
pub const AMMUNITION: Source = Source {
    folder: "combat/ammunition",
    file: "Ammunition.ndf",
};
pub const MISSILES: Source = Source {
    folder: "combat/ammunition-missiles",
    file: "AmmunitionMissiles.ndf",
};
pub const FLIGHT: Source = Source {
    folder: "combat/missiles",
    file: "MissileDescriptors.ndf",
};

pub const CUSTOM: Source = Source {
    folder: "compatibility/ammunition",
    file: "Ammunition.ndf",
};

pub struct Rule {
    pub source: Source,
    pub name: String,
    pub operation: Value,
    pub matcher: ndf::operation::BulkMatch,
}

pub fn parse(source: Source, text: &str) -> Result<Vec<Rule>> {
    let value: Value = serde_json::from_str(text).map_err(|error| {
        failure(
            format!("Invalid {} rules: {error}", source.folder),
            "Correct the feature's rules.json.",
        )
    })?;
    let values = ymb_sdk::values::record(&value, "rules")?;
    values
        .iter()
        .map(|(name, operation)| {
            ensure(
                operation
                    .get("edits")
                    .and_then(Value::as_array)
                    .is_some_and(|edits| !edits.is_empty() && edits.iter().all(Value::is_object)),
                format!("Rule {name} needs a nonempty edits list containing objects."),
                "Correct the rule's edits in rules.json.",
            )?;
            let matcher = serde_json::from_value(operation["match"].clone()).map_err(|error| {
                failure(
                    format!("Invalid rule {name}: {error}"),
                    "Correct its bulk match conditions.",
                )
            })?;
            Ok(Rule {
                source,
                name: name.clone(),
                operation: operation.clone(),
                matcher,
            })
        })
        .collect()
}

pub fn load(ctx: &mut Context, source: Source) -> Result<Vec<Rule>> {
    parse(
        source,
        &ctx.read_mod_text(&format!("patch/{}/rules.json", source.folder))?,
    )
}

pub fn generate(ctx: &mut Context, source: Source) -> Result<Vec<GeneratedFile>> {
    let rules = load(ctx, source)?;
    let file = format!("{GFX}/{}", source.file);
    let text = ctx.read_target(&file)?;
    let allow_missing = ctx
        .variables
        .pointer("/compatibility/allowMissingVanillaTargets")
        .unwrap_or(&Value::Bool(false))
        .as_bool()
        .ok_or_else(|| {
            failure(
                "allowMissingVanillaTargets must be boolean.",
                "Use true or false.",
            )
        })?;
    let blocks = if allow_missing {
        ndf::scan(&text)?
    } else {
        Vec::new()
    };
    let mut operations = Vec::new();
    for rule in &rules {
        let mut operation = rule.operation.clone();
        if allow_missing {
            let mut count = 0;
            for block in &blocks {
                if ndf::patch::matches(
                    &text[block.span.clone()],
                    block.name,
                    block.type_name,
                    &rule.matcher,
                )? {
                    count += 1;
                }
            }
            if count == 0 {
                ctx.notice(format!(
                    "Compatibility skipped absent {} rule {}.",
                    source.folder, rule.name
                ))?;
            }
            operation["expect"] = serde_json::json!({"minBlocks":0});
            operation["allowNoChange"] =
                serde_json::json!("compatibility: absent vanilla content is explicitly allowed.");
            for edit in operation["edits"].as_array_mut().expect("validated edits") {
                edit.as_object_mut()
                    .expect("validated edit")
                    .remove("minChanges");
            }
        }
        operations.push(operation);
    }
    let result = ctx.patch(&text, &file, &operations)?;
    let text = checked_text(result, &rules)?;
    Ok(vec![ctx.output_text(&file, text)?])
}

fn checked_text(result: ndf::patch::PatchResult, rules: &[Rule]) -> Result<String> {
    let errors = result
        .notices
        .iter()
        // WTO requires every rule to apply unless compatibility explicitly permits a no-op.
        .filter(|notice| notice.expected.is_none())
        .map(|notice| {
            *ymb_sdk::Diagnostic::new("GeneratorError", &notice.reason, &notice.suggestion)
                .code(match notice.code.as_str() {
                    "W8" => "E18",
                    "W9" => "E19",
                    code => code,
                })
                .at(&notice.path)
                .with("operationIndex", notice.operation_index)
                .with("rule", &rules[notice.operation_index].name)
        })
        .collect();
    if let Some(error) = ymb_sdk::Diagnostic::group(errors) {
        return Err(error);
    }
    Ok(result.text)
}

#[test]
fn rule_findings_are_reported_instead_of_being_discarded() {
    let operation = serde_json::json!({"op":"bulk","match":{"conditions":[{"on":"name","is":"equals","value":["Existing","Misspelled"]}]},"edits":[{"field":"Value","set":2}]});
    let rules = parse(
        AMMUNITION,
        &serde_json::json!({"range":operation}).to_string(),
    )
    .unwrap();
    let operations =
        ndf::operation::expand(&[rules[0].operation.clone()], &serde_json::json!({})).unwrap();
    let source = "Existing is T(Value = 1)";
    let result = ndf::patch::apply(source, &operations, &Default::default()).unwrap();
    let error = checked_text(result, &rules).unwrap_err();
    assert_eq!(error.context["rule"], "range");
    assert_eq!(error.context["operationIndex"], 0);
    assert!(error.reason.contains("Misspelled"));
    let mut expected = rules[0].operation.clone();
    expected["allowNoChange"] = serde_json::json!("Compatibility with an absent optional role.");
    let operations = ndf::operation::expand(&[expected], &serde_json::json!({})).unwrap();
    let result = ndf::patch::apply(source, &operations, &Default::default()).unwrap();
    assert!(checked_text(result, &rules).unwrap().contains("Value = 2"));
}

#[test]
fn malformed_edits_are_reported_before_compatibility_can_mutate_them() {
    for edits in [
        Value::Null,
        serde_json::json!([]),
        serde_json::json!([null]),
        serde_json::json!("wrong"),
    ] {
        let text = serde_json::json!({"broken":{"match":{"conditions":[{"on":"name","is":"equals","value":"A"}]},"edits":edits}}).to_string();
        let error = parse(AMMUNITION, &text).err().unwrap();
        assert!(error.reason.contains("broken"));
        assert!(error.reason.contains("edits"));
    }
}

#[test]
fn inherited_bmp2_timing_keeps_the_rest_of_the_rebalance_and_required_counts() {
    let rules = parse(
        AMMUNITION,
        include_str!("../../patch/combat/ammunition/rules.json"),
    )
    .unwrap();
    let rule = rules
        .into_iter()
        .find(|rule| rule.name == "bmp2Ap")
        .unwrap();
    let operations = ndf::operation::expand(
        std::slice::from_ref(&rule.operation),
        &serde_json::json!({
            "modTag":"Fixture", "multipliers":{"timing":5,"supply":1,"ammunition":1,"dispersion":1},
            "tuning":{"bmp2":{"salvoSize":13,"salvoMultiplier":1,"accuracyMultiplier":1}}
        }),
    )
    .unwrap();
    let source = "Ammo_AutoCanon_AP_30mm_24A2_BMP_Fixture is TAmmunitionDescriptor(\n\
        TimeBetweenTwoShots = 1.0 TimeBetweenTwoFx = 1.0 ShotsCountPerSalvo = 3\n\
        AffichageMunitionParSalve = 3 SupplyCost = 3 ShotsBeforeMaxNoise = 3\n\
        DispersionAtMaxRangeGRU = 3 BaseHitValueModifiers = [(EBaseHitValueModifier/Idling, 2), (EBaseHitValueModifier/Moving, 2)])";
    let result = ndf::patch::apply(source, &operations, &Default::default()).unwrap();
    assert_eq!(result.notices.len(), 2);
    assert!(result.notices.iter().all(|notice| notice.code == "I4"));
    let output = checked_text(result, &[rule]).unwrap();
    assert!(output.contains("ShotsCountPerSalvo = 13"));
    assert!(output.contains("TimeBetweenTwoShots = 1.0"));
    assert!(
        ndf::patch::apply(
            &source.replace("TimeBetweenTwoShots = 1.0", ""),
            &operations,
            &Default::default()
        )
        .is_err()
    );
}
