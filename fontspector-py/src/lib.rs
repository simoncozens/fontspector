#![deny(clippy::unwrap_used, clippy::expect_used)]
use std::{env, path::Path, vec};
// Provide an environment where we can run fontbakery tests
// as-is, but have them call a Rust implementation underneath
use fontspector_checkapi::{
    Context, Plugin, Registry, StatusCode, Testable, TestableCollection, TestableType,
};
use profile_googlefonts::GoogleFonts;
use profile_opentype::OpenType;
use profile_universal::Universal;
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyDictMethods, PyList, PyString, PyTuple},
};

#[pyclass]
struct CheckTester(String);

fn obj_to_testable(py: Python, arg: &Bound<'_, PyAny>) -> PyResult<Testable> {
    let ttfont_class = py.import_bound("fontTools.ttLib")?.getattr("TTFont")?;
    // if it's a string, just return a new testable
    if arg.is_instance_of::<PyString>() {
        let filename: String = arg.extract()?;
        return Testable::new(&filename)
            .map_err(|e| PyValueError::new_err(format!("Couldn't create testable object: {}", e)));
    }
    if !arg.is_instance(&ttfont_class)? {
        panic!("I can't handle args {:?}", arg);
    }
    let filename: String = arg
        .getattr("reader")?
        .getattr("file")?
        .getattr("name")?
        .extract()?;
    let basename = Path::new(&filename)
        .file_name()
        .ok_or_else(|| PyValueError::new_err("Couldn't extract basename from filename"))?;
    let tempfile = env::temp_dir().join(basename);
    let tempfile = tempfile
        .to_str()
        .ok_or_else(|| PyValueError::new_err("Couldn't convert tempfile path to string"))?;
    arg.call_method1("save", (tempfile,))?;
    let testable = Testable::new(tempfile)
        .map_err(|e| PyValueError::new_err(format!("Couldn't create testable object: {}", e)))?;
    Ok(testable)
}

#[pymethods]
impl CheckTester {
    #[new]
    fn new(check: String) -> Self {
        Self(check)
    }

    #[pyo3(signature = (*args, **kwargs))]
    fn __call__<'a>(
        &self,
        py: Python<'a>,
        args: &Bound<'a, PyTuple>,
        kwargs: Option<&Bound<'a, PyDict>>,
    ) -> PyResult<Vec<Bound<'a, PyAny>>> {
        // Spin up a new fontspector (each time, how extravagant)
        let mut registry = Registry::new();
        OpenType.register(&mut registry).map_err(|_| {
            PyValueError::new_err("Couldn't register opentype profile, fontspector bug")
        })?;
        Universal.register(&mut registry).map_err(|_| {
            PyValueError::new_err("Couldn't register universal profile, fontspector bug")
        })?;
        GoogleFonts.register(&mut registry).map_err(|_| {
            PyValueError::new_err("Couldn't register Google Fonts profile, fontspector bug")
        })?;

        let check = registry
            .checks
            .get(&self.0)
            .ok_or_else(|| PyValueError::new_err("Check not found"))?;

        // We have almost certainly been handed a TTFont object. Turn it into a testable
        let first_arg = args
            .get_item(0)
            .map_err(|_| PyValueError::new_err("No args found"))?;
        let testables = if first_arg.is_instance_of::<PyList>() {
            first_arg
                .iter()?
                .flatten()
                .map(|a| obj_to_testable(py, &a))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            vec![obj_to_testable(py, &first_arg)?]
        };
        let collection = TestableCollection { testables };
        let newargs = if collection.testables.len() == 1 {
            TestableType::Single(&collection.testables[0])
        } else {
            TestableType::Collection(&collection)
        };

        let mut fontspector_config = serde_json::Map::new();

        if let Some(kwargs) = kwargs {
            if let Some(config) = kwargs.get_item("config")? {
                // Ideally we should convert the whole PyDict into a serde_json::Value
                // but YAGNI.
                let config = config.downcast::<PyDict>()?;
                for (k, v) in config {
                    let k = k.downcast::<PyString>()?.to_string();
                    let v = v.downcast::<PyString>()?.to_string();
                    fontspector_config.insert(k, serde_json::Value::String(v));
                }
            }
        }

        let context = Context {
            configuration: fontspector_config,
            ..Default::default()
        };

        // Run the check!
        let result = check
            .run(&newargs, &context, None)
            .ok_or_else(|| PyValueError::new_err("No results returned?"))?;
        // Map results back to a Python list of subresults
        let status_module = py.import_bound("fontbakery.status")?;
        let subresult_module = py.import_bound("fontbakery.result")?;
        let message_class = py.import_bound("fontbakery.message")?.getattr("Message")?;
        let mut py_subresults = vec![];
        for subresult in result.subresults {
            let severity = match subresult.severity {
                StatusCode::Skip => status_module.getattr("SKIP")?,
                StatusCode::Info => status_module.getattr("INFO")?,
                StatusCode::Warn => status_module.getattr("WARN")?,
                StatusCode::Pass => status_module.getattr("PASS")?,
                StatusCode::Fail => status_module.getattr("FAIL")?,
                StatusCode::Error => status_module.getattr("ERROR")?,
            };
            let message = message_class.call1((
                subresult.code.unwrap_or("None".to_string()),
                subresult.message.unwrap_or("No message".to_string()),
            ))?;
            py_subresults.push(
                subresult_module
                    .getattr("Subresult")?
                    .call1((severity, message))?,
            )
        }
        Ok(py_subresults)
    }
}

#[pyfunction]
fn registered_checks() -> PyResult<Vec<String>> {
    let mut registry = Registry::new();
    OpenType.register(&mut registry).map_err(|_| {
        PyValueError::new_err("Couldn't register opentype profile, fontspector bug")
    })?;
    Universal.register(&mut registry).map_err(|_| {
        PyValueError::new_err("Couldn't register universal profile, fontspector bug")
    })?;
    GoogleFonts.register(&mut registry).map_err(|_| {
        PyValueError::new_err("Couldn't register Google Fonts profile, fontspector bug")
    })?;
    Ok(registry.checks.keys().cloned().collect())
}

#[pymodule(name = "fontspector")]
fn fonspector(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CheckTester>()?;
    m.add_function(wrap_pyfunction!(registered_checks, m)?)
}
