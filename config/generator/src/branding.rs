use serde_json::Value;
use std::collections::{HashMap, HashSet};
use ymb_sdk::localisation::{cell, row};
use ymb_sdk::{Context, GeneratedFile, Result, TestResult, ensure, values};

fn apply(content: &str, replacements: &HashMap<String, String>) -> Result<Option<String>> {
    let newline = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut tokens = HashSet::new();
    let mut count = 0;
    let normalized = content.replace("\r\n", "\n");
    let lines: Vec<_> = normalized
        .split('\n')
        .map(|line| {
            let Some((token, _)) = row(line) else {
                return line.to_owned();
            };
            let Some(text) = replacements.get(&token) else {
                return line.to_owned();
            };
            count += 1;
            let line = format!("{};{}", cell(&token), cell(text));
            tokens.insert(token);
            line
        })
        .collect();
    if count == 0 {
        return Ok(None);
    }
    ensure(
        count == replacements.len() && tokens.len() == replacements.len(),
        "The YSM dictionary has missing or duplicate branding rows.",
        "Update YSM and WTO together, then check the configured branding tokens.",
    )?;
    Ok(Some(lines.join(newline)))
}

pub fn generate(ctx: &mut Context) -> Result<Vec<GeneratedFile>> {
    let value: Value = ctx.value("branding")?;
    let replacements: HashMap<_, _> = values::record(&value, "branding")?
        .iter()
        .map(|(token, value)| Ok((token.clone(), values::string(value, token)?.to_owned())))
        .collect::<Result<_>>()?;
    let root: String = ctx.value("modRootName")?;
    let path = format!("GameData/Localisation/{root}/INTERFACE_OUTGAME.csv");
    let source = match ctx.read_target(&path) {
        Ok(source) => source,
        Err(error) if error.code.as_deref() == Some("E8") => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    match apply(&source, &replacements)? {
        Some(text) => Ok(vec![ctx.output_text(&path, text)?]),
        None => Ok(Vec::new()),
    }
}

pub fn checks(_: &mut Context) -> Result<Vec<TestResult>> {
    check()?;
    Ok(vec![TestResult::passed(
        "Branding preserves CSV quoting and rejects partial dictionaries",
    )])
}

fn check() -> Result<()> {
    let replacements = HashMap::from([
        ("A".into(), "One".into()),
        ("B".into(), "Line 1\r\n\"Line 2\"".into()),
    ]);
    let source =
        "\"TOKEN\";\"REFTEXT\"\r\n\"A\";\"Before\"\r\n\"B\";\"Before\"\r\n\"OTHER\";\"Keep\"\r\n";
    let changed = apply(source, &replacements)?.unwrap();
    ensure(
        changed.contains("\"B\";\"Line 1\\n\"\"Line 2\"\"\"\r\n")
            && changed.contains("\"OTHER\";\"Keep\"\r\n")
            && apply(&changed, &replacements)?.as_ref() == Some(&changed)
            && apply("\"OTHER\";\"Keep\"\n", &replacements)?.is_none()
            && apply(&source.replace("\"B\"", "\"MISSING\""), &replacements).is_err()
            && apply(&format!("{source}\"A\";\"Duplicate\"\r\n"), &replacements).is_err(),
        "Branding changed unrelated text or accepted an incomplete dictionary.",
        "Keep token counts, quoting, and line endings intact.",
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn branding_handles_quoting_newlines_and_incomplete_dictionaries() {
        super::check().unwrap();
    }
}
