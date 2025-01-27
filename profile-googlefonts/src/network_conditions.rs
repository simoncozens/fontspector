use fontspector_checkapi::Context;
use serde_json::{Map, Value};

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

pub(crate) fn production_metadata(context: &Context) -> Result<Map<String, Value>, String> {
    #[cfg(not(target_family = "wasm"))]
    {
        if context.skip_network {
            return Err("Network access disabled".to_string());
        }
        PRODUCTION_METADATA.clone()
    }
    #[cfg(target_family = "wasm")]
    {
        Err("Network access disabled".to_string())
    }
}

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
                f.get("family")
                    .and_then(Value::as_str) == Some(family)
            }))
        },
        Value::Bool,
        |v| v.as_bool().ok_or("Expected a boolean".to_string()),
    )
}
