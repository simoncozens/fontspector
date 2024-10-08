#![allow(non_upper_case_globals)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

use fontspector_checkapi::{prelude::*, StatusCode};
use pyo3::{
    prelude::*,
    types::{PyList, PyTuple},
};
use serde_json::json;
pub struct FontbakeryBridge;

// We isolate the Python part to avoid type/result madness.
fn python_checkrunner_impl(
    module: &str,
    function: &str,
    testable: &Testable,
) -> PyResult<CheckFnResult> {
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

        let mut checkresult = check.call1((arg,))?;
        let mut messages: Vec<Status> = vec![];

        // If the checkresult is a single tuple, turn it into a list of tuples, and get a generator
        if checkresult.is_instance_of::<PyTuple>() {
            let checkresults = vec![checkresult];
            checkresult = PyList::new_bound(py, checkresults)
                .getattr("__iter__")?
                .call0()?;
        }

        // Now convert the Fontbakery status to our StatusList
        while let Ok(value) = checkresult.getattr("__next__")?.call0() {
            // Value is a tuple of status and message
            let status_str = value.get_item(0)?.getattr("name")?.extract::<String>()?;
            let status = StatusCode::from_string(&status_str).ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Fontbakery returned unknown status code".to_string(),
                )
            })?;
            let code = if value.get_item(1)?.hasattr("code")? {
                Some(value.get_item(1)?.getattr("code")?.extract::<String>()?)
            } else {
                None
            };
            let message = if value.get_item(1)?.hasattr("message")? {
                value.get_item(1)?.getattr("message")?
            } else {
                value.get_item(1)?
            }
            .extract::<String>()?;

            messages.push(Status {
                message: Some(message),
                severity: status,
                code,
            });
        }
        Ok(return_result(messages))
    })
}

// This wrapper will work for any fontbakery check that takes a single
// Font or ttFont object as an argument.
fn python_checkrunner(c: &Testable, context: &Context) -> CheckFnResult {
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
    python_checkrunner_impl(module, function, c)
        .unwrap_or_else(|e| Err(CheckError::Error(format!("Python error: {}", e))))
}

fn register_python_checks(modulename: &str, source: &str, cr: &mut Registry) -> Result<(), String> {
    Python::with_gil(|py| {
        // Assert that we have loaded the FB prelude
        let _prelude = PyModule::import_bound(py, "fontbakery.prelude")?;
        let callable = PyModule::import_bound(py, "fontbakery.callable")?;
        let full_source = "from fontbakery.prelude import *\n\n".to_string() + source;
        let module =
            PyModule::from_code_bound(py, &full_source, &format!("{}.py", modulename), modulename)?;
        // let check = module.getattr("check_hinting_impact")?;
        // Find all functions in the module which are checks
        let checktype = callable.getattr("FontBakeryCheck")?;
        for name in module.dir()?.iter() {
            let name_str: String = name.extract()?;
            let obj = module.getattr(name.downcast()?)?;
            if let Ok(true) = obj.is_instance(&checktype) {
                let id: String = obj.getattr("id")?.extract()?;
                // Check the mandatory arguments
                let args = obj.getattr("mandatoryArgs")?.extract::<Vec<String>>()?;
                if args.len() != 1 || !(args[0] == "font" || args[0] == "ttFont") {
                    log::warn!(
                        "Can't load check {}; unable to support arguments: {}",
                        id,
                        args.join(", ")
                    );
                    continue;
                }
                let title: String = obj.getattr("__doc__")?.extract()?;
                let py_rationale = obj.getattr("rationale")?;
                let rationale: String = if py_rationale.is_instance_of::<PyList>() {
                    let r: Vec<String> = py_rationale.extract()?;
                    r.join(", ")
                } else {
                    py_rationale.extract()?
                };
                let py_proposal = obj.getattr("proposal")?;
                let proposal: String = if py_proposal.is_instance_of::<PyList>() {
                    let r: Vec<String> = py_proposal.extract()?;
                    r.join(", ")
                } else {
                    py_proposal.extract()?
                };
                log::info!("Registered check: {}", id);
                let metadata = json!({
                    "module": modulename,
                    "function": name_str,
                });
                cr.register_check(Check {
                    id: id.leak(),
                    title: title.leak(),
                    rationale: rationale.leak(),
                    proposal: proposal.leak(),
                    hotfix: None,
                    fix_source: None,
                    applies_to: "TTF",
                    flags: CheckFlags::default(),
                    implementation: CheckImplementation::CheckOne(&python_checkrunner),
                    _metadata: Some(metadata.to_string().leak()),
                })
            }
        }
        Ok(())
    })
    .map_err(|e: PyErr| format!("Error loading checks: {}", e))
}

impl fontspector_checkapi::Plugin for FontbakeryBridge {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        pyo3::prepare_freethreaded_python();
        // Load needed FB modules
        let ok: PyResult<()> = Python::with_gil(|py| {
            PyModule::from_code_bound(
                py,
                include_str!("../fontbakery/Lib/fontbakery/callable.py"),
                "callable.py",
                "fontbakery.callable",
            )?;
            PyModule::from_code_bound(
                py,
                include_str!("../fontbakery/Lib/fontbakery/status.py"),
                "status.py",
                "fontbakery.status",
            )?;
            PyModule::from_code_bound(
                py,
                include_str!("../fontbakery/Lib/fontbakery/message.py"),
                "message.py",
                "fontbakery.message",
            )?;
            Ok(())
        });
        ok.map_err(|e| format!("Error loading FB modules: {}", e))?;

        register_python_checks(
            "fontbakery.checks.opentype.kern",
            include_str!("../fontbakery/Lib/fontbakery/checks/opentype/kern.py"),
            cr,
        )?;
        register_python_checks(
            "fontbakery.checks.opentype.cff",
            include_str!("../fontbakery/Lib/fontbakery/checks/opentype/cff.py"),
            cr,
        )?;
        register_python_checks(
            "fontbakery.checks.opentype.fvar",
            include_str!("../fontbakery/Lib/fontbakery/checks/opentype/fvar.py"),
            cr,
        )?;
        register_python_checks(
            "fontbakery.checks.opentype.gdef",
            include_str!("../fontbakery/Lib/fontbakery/checks/opentype/gdef.py"),
            cr,
        )?;
        register_python_checks(
            "fontbakery.checks.opentype.gpos",
            include_str!("../fontbakery/Lib/fontbakery/checks/opentype/gpos.py"),
            cr,
        )?;
        register_python_checks(
            "fontbakery.checks.opentype.head",
            include_str!("../fontbakery/Lib/fontbakery/checks/opentype/head.py"),
            cr,
        )?;
        register_python_checks(
            "fontbakery.checks.opentype.hhea",
            include_str!("../fontbakery/Lib/fontbakery/checks/opentype/hhea.py"),
            cr,
        )?;
        register_python_checks(
            "fontbakery.checks.opentype.os2",
            include_str!("../fontbakery/Lib/fontbakery/checks/opentype/os2.py"),
            cr,
        )?;
        register_python_checks(
            "fontbakery.checks.some_other_checks",
            include_str!("../fontbakery/Lib/fontbakery/checks/some_other_checks.py"),
            cr,
        )?;
        register_python_checks(
            "fontbakery.checks.glyphset",
            include_str!("../fontbakery/Lib/fontbakery/checks/glyphset.py"),
            cr,
        )?;
        register_python_checks(
            "fontbakery.checks.metrics",
            include_str!("../fontbakery/Lib/fontbakery/checks/metrics.py"),
            cr,
        )?;
        register_python_checks(
            "fontbakery.checks.hinting",
            include_str!("../fontbakery/Lib/fontbakery/checks/hinting.py"),
            cr,
        )?;
        cr.register_profile(
            "fontbakery",
            Profile::from_toml(
                r#"
        [sections]
        "Test profile" = [
            "hinting_impact",
            "opentype/name/empty_records",
            "opentype/monospace",
            "opentype/cff_call_depth",
        ]
        "#,
            )
            .map_err(|_| "Couldn't parse profile")?,
        )
    }
}

#[cfg(not(target_family = "wasm"))]
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, FontbakeryBridge);
