use super::*;
use serde_json::json;
use ymb_sdk::{ensure, failure};

pub(super) fn rule_index(rules: &[Rule], source: rules::Source, name: &str) -> Result<usize> {
    rules
        .iter()
        .position(|rule| rule.source == source && rule.name == name)
        .ok_or_else(|| {
            failure(
                format!("Missing {} rule {name}.", source.folder),
                "Restore the feature's rules.json.",
            )
        })
}

pub(super) fn objects(
    content: &str,
    path: &str,
    selected: &[(&ndf::Block<'_>, Vec<usize>)],
    rules: &[Rule],
    variables: &Value,
    identity: &ndf::operation::Identity,
) -> Result<String> {
    if selected.is_empty() {
        return Ok(content.to_owned());
    }
    let mut subjects = String::new();
    let mut groups: Vec<(usize, Vec<String>)> = Vec::new();
    for (index, (block, indices)) in selected.iter().enumerate() {
        let body = block.body.as_ref().ok_or_else(|| {
            failure(
                "Selected ammunition has no body.",
                "Check the object's NDF syntax.",
            )
        })?;
        subjects.push_str(&format!(
            "Subject{index} is {}\n{}\n",
            block.type_name,
            &content[body.clone()]
        ));
        let mut seen = HashSet::new();
        for rule in indices.iter().copied().filter(|rule| seen.insert(*rule)) {
            if let Some((_, names)) = groups.iter_mut().find(|(id, _)| *id == rule) {
                names.push(format!("Subject{index}"));
            } else {
                groups.push((rule, vec![format!("Subject{index}")]));
            }
        }
    }
    let operations: Vec<_> = groups.into_iter().map(|(rule, names)| {
        let mut edits = rules[rule].operation["edits"].clone();
        if let Some(edits) = edits.as_array_mut() { for edit in edits { edit.as_object_mut().map(|edit| edit.remove("minChanges")); } }
        json!({"op":"bulk","match":{"conditions":[{"on":"name","is":"equals","value":names}]},"expect":{"minBlocks":names.len()},"edits":edits})
    }).collect();
    let identity = ndf::operation::Identity {
        path: path.into(),
        ..identity.clone()
    };
    let result = ndf::patch::apply(
        &subjects,
        &ndf::operation::expand(&operations, variables)?,
        &identity,
    )?;
    let blocks = ndf::scan(&result.text)?;
    ensure(
        blocks.len() == selected.len(),
        "Custom ammunition changed its object structure.",
        "Check the selected rules and object bodies.",
    )?;
    let mut output = String::with_capacity(content.len());
    let mut cursor = 0;
    for (index, ((original, _), changed)) in selected.iter().zip(blocks).enumerate() {
        let body = original.body.as_ref().unwrap();
        ensure(
            changed.name == Some(format!("Subject{index}").as_str()) && body.start >= cursor,
            "Selected custom ammunition overlaps or is missing.",
            "Check the generated object structure.",
        )?;
        output.push_str(&content[cursor..body.start]);
        output.push_str(&result.text[changed.body.unwrap()]);
        cursor = body.end;
    }
    output.push_str(&content[cursor..]);
    Ok(output)
}
