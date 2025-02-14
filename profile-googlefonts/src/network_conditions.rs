use fontspector_checkapi::{Context, Testable};
#[allow(unused_imports)]
use serde_json::{json, Map, Value};

#[cfg(not(target_family = "wasm"))]
pub(crate) static PRODUCTION_METADATA: std::sync::LazyLock<Result<Map<String, Value>, String>> =
    std::sync::LazyLock::new(|| {
        reqwest::blocking::get("https://fonts.google.com/metadata/fonts")
            .map_err(|e| format!("Failed to fetch metadata: {}", e))
            .and_then(|response| {
                response.text().map_or_else(
                    |e| Err(format!("Failed to fetch metadata: {}", e)),
                    |s| {
                        serde_json::from_str(&s)
                            .map_err(|e| format!("Failed to parse metadata: {}", e))
                    },
                )
            })
    });

#[allow(dead_code)]
pub(crate) fn production_metadata(context: &Context) -> Result<Map<String, Value>, String> {
    if context.skip_network {
        return Err("Network access disabled".to_string());
    }
    #[cfg(not(target_family = "wasm"))]
    {
        PRODUCTION_METADATA.clone()
    }
    #[cfg(target_family = "wasm")]
    {
        Err("Network access disabled".to_string())
    }
}

#[allow(dead_code)]
pub(crate) fn is_listed_on_google_fonts(family: &str, context: &Context) -> Result<bool, String> {
    // println!("Looking for family {}", family);
    let key = format!("is_listed_on_google_fonts:{}", family);
    context.cached_question(
        &key,
        || {
            let metadata = production_metadata(context)?;
            let family_metadata_list = metadata
                .get("familyMetadataList")
                .ok_or("Failed to find familyMetadataList in production metadata".to_string())?
                .as_array()
                .ok_or("familyMetadataList is not an object".to_string())?;
            Ok(family_metadata_list.iter().any(|f| {
                // println!("Looking at family {:?}", f.get("family"));
                f.get("family").and_then(Value::as_str) == Some(family)
            }))
        },
        Value::Bool,
        |v| v.as_bool().ok_or("Expected a boolean".to_string()),
    )
}

#[allow(unused_variables)]
pub(crate) fn remote_styles(family: &str, context: &Context) -> Result<Vec<Testable>, String> {
    #[cfg(target_family = "wasm")]
    {
        Err("Network access disabled".to_string())
    }
    #[cfg(not(target_family = "wasm"))]
    {
        remote_styles_impl(family, context)
    }
}

#[cfg(not(target_family = "wasm"))]
fn remote_styles_impl(family: &str, context: &Context) -> Result<Vec<Testable>, String> {
    let key = format!("remote_styles:{}", family);
    context.cached_question(
        &key,
        || {
            let mut request = reqwest::blocking::Client::new().get(format!(
                "https://fonts.google.com/download/list?family={}",
                family.replace(" ", "%20")
            ));
            if let Some(timeout) = context.network_timeout {
                request = request.timeout(std::time::Duration::new(timeout, 0));
            }
            let manifest: serde_json::Value = request
                .send()
                .and_then(|response| response.text())
                .map_or_else(
                |e| Err(format!("Failed to fetch metadata: {}", e)),
                |s| {
                    serde_json::from_str(&s[5..])
                        .map_err(|e| format!("Failed to parse metadata: {}", e))
                },
            )?;
            let mut fonts = vec![];
            for file in manifest
                .as_object()
                .and_then(|x| x.get("manifest"))
                .and_then(|x| x.as_object())
                .and_then(|x| x.get("fileRefs"))
                .and_then(|x| x.as_array())
                .ok_or("Failed to find fileRefs in manifest".to_string())?
            {
                let url = file
                    .as_object()
                    .and_then(|x| x.get("url"))
                    .and_then(|x| x.as_str())
                    .ok_or("Failed to find url in file".to_string())?;
                let filename = file
                    .as_object()
                    .and_then(|x| x.get("filename"))
                    .and_then(|x| x.as_str())
                    .ok_or("Failed to filename url in file".to_string())?;
                if filename.contains("static")
                    || !filename.ends_with("otf") && !filename.ends_with("ttf")
                {
                    continue;
                }
                let contents = reqwest::blocking::get(url)
                    .map_err(|e| format!("Failed to fetch font: {}", e))?
                    .bytes()
                    .map_err(|e| format!("Failed to fetch font: {}", e))?;
                let testable = Testable::new_with_contents(filename, contents.to_vec());
                fonts.push(testable);
            }
            Ok(fonts)
        },
        |testables| {
            Value::Array(
                testables
                    .iter()
                    .map(|t| {
                        json!({
                                "filename": t.filename.to_str().unwrap_or_default().to_string(),
                                "contents": t.contents,
                        })
                    })
                    .collect(),
            )
        },
        |v| {
            v.as_array()
                .ok_or("Expected an array".to_string())
                .and_then(|a| {
                    a.iter()
                        .map(|v| {
                            let filename = v
                                .get("filename")
                                .and_then(Value::as_str)
                                .ok_or("Expected a string".to_string())?;
                            let contents = v
                                .get("contents")
                                .and_then(Value::as_str)
                                .ok_or("Expected a string".to_string())?;
                            Ok(Testable::new_with_contents(
                                filename,
                                contents.as_bytes().to_vec(),
                            ))
                        })
                        .collect()
                })
        },
    )
}
