#![allow(non_upper_case_globals)]
#![deny(clippy::unwrap_used, clippy::expect_used)]
use fontspector_checkapi::{prelude::*, StatusCode};
use pyo3::prelude::*;
mod checks;
struct FontbakeryBridge;

// We isolate the Python part to avoid type/result madness.
fn call_python(module: &str, function: &str, testable: &Testable) -> PyResult<CheckFnResult> {
    let filename = testable.filename.to_string_lossy();
    Python::with_gil(|py| {
        let module = PyModule::import_bound(py, module)?;
        let check = module.getattr(function)?;

        // Let's check this check's mandatory arguments
        let args = check.getattr("mandatoryArgs")?.extract::<Vec<String>>()?;
        if args.len() != 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Expected exactly one mandatory argument".to_string(),
            ));
        }
        let arg = if args[0] == "font" {
            // Convert the Testable to a Python Font object
            let testable = PyModule::import_bound(py, "fontbakery.testable")?;
            let font = testable.getattr("Font")?;
            font.call1((filename,))?
        } else if args[0] == "ttFont" {
            // Convert the Testable to a Python TTFont object
            let ttlib = PyModule::import_bound(py, "fontTools.ttLib")?;
            let ttfont = ttlib.getattr("TTFont")?;
            ttfont.call1((filename,))?
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Unknown mandatory argument".to_string(),
            ));
        };

        let checkresult = check.call1((arg,))?;
        let mut messages: Vec<Status> = vec![];

        // Now convert the Fontbakery status to our StatusList
        while let Ok(value) = checkresult.getattr("__next__")?.call0() {
            // Value is a tuple of status and message
            let status_str = value.get_item(0)?.getattr("name")?.extract::<String>()?;
            let status = StatusCode::from_string(&status_str).ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Fontbakery returned unknown status code".to_string(),
                )
            })?;
            let code = value.get_item(1)?.getattr("code")?.extract::<String>()?;
            let message = value.get_item(1)?.getattr("message")?.extract::<String>()?;
            messages.push(Status {
                message: Some(message),
                severity: status,
                code: Some(code),
            });
        }
        Ok(return_result(messages))
    })
}

// This wrapper will work for any fontbakery check that takes a single
// Font or ttFont object as an argument.
fn run_a_python_test(c: &Testable, context: &Context) -> CheckFnResult {
    let module = context
        .check_metadata
        .get("module")
        .ok_or_else(|| CheckError::Error("No module specified".to_string()))?
        .as_str()
        .ok_or_else(|| CheckError::Error("module in metadata was not a string!".to_string()))?;
    let function = context
        .check_metadata
        .get("function")
        .ok_or_else(|| CheckError::Error("No function specified".to_string()))?
        .as_str()
        .ok_or_else(|| CheckError::Error("function in metadata was not a string!".to_string()))?;
    call_python(module, function, c)
        .unwrap_or_else(|e| Err(CheckError::Error(format!("Python error: {}", e))))
}

impl fontspector_checkapi::Plugin for FontbakeryBridge {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        cr.register_check(checks::hinting_impact);
        cr.register_check(checks::opentype_name_empty_records);
        cr.register_check(checks::monospace);
        pyo3::prepare_freethreaded_python();
        cr.register_profile(
            "fontbakery",
            Profile::from_toml(
                r#"
[sections]
"Test profile" = [
    "hinting_impact",
    "opentype/name/empty_records",
    "opentype/monospace",
]
"#,
            )
            .map_err(|_| "Couldn't parse profile")?,
        )
    }
}

#[cfg(not(target_family = "wasm"))]
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, FontbakeryBridge);
