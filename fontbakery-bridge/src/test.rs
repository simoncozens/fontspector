#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
use std::{
    env,
    path::{Path, PathBuf},
    sync::Mutex,
};

// Provide an environment where we can run fontbakery tests
// as-is, but have them call a Rust implementation underneath
use fontspector_checkapi::{Context, Plugin, Registry, StatusCode, Testable};
use profile_googlefonts::GoogleFonts;
use profile_opentype::OpenType;
use profile_universal::Universal;
use pyo3::{
    prelude::*,
    types::{PyCFunction, PyList, PyTuple},
};

#[pyclass]
struct CheckTester(String);

#[pyfunction]
fn checktester(check: &str) -> PyResult<Py<CheckTester>> {
    Python::with_gil(|py| Py::new(py, CheckTester(check.to_string())))
}

#[pymethods]
impl CheckTester {
    #[pyo3(signature = (*args))]
    fn __call__<'a>(
        &self,
        py: Python<'a>,
        args: &Bound<'a, PyTuple>,
    ) -> PyResult<Vec<Bound<'a, PyAny>>> {
        let ttfont_class = py.import_bound("fontTools.ttLib")?.getattr("TTFont")?;

        // Spin up a new fontspector (each time, how extravagant)
        let mut registry = Registry::new();
        OpenType
            .register(&mut registry)
            .expect("Couldn't register opentype profile, fontspector bug");
        Universal
            .register(&mut registry)
            .expect("Couldn't register universal profile, fontspector bug");
        GoogleFonts.register(&mut registry).unwrap();

        let check = registry.checks.get(&self.0).expect("Check not found");

        // We have almost certainly been handed a TTFont object. Turn it into a testable
        if args.len() != 1 || !args.get_item(0)?.is_instance(&ttfont_class)? {
            panic!("I can't handle args {:?}", args);
        }
        let ttfont = args.get_item(0)?;
        let filename: String = ttfont
            .getattr("reader")?
            .getattr("file")?
            .getattr("name")?
            .extract()?;
        let basename = Path::new(&filename).file_name().unwrap();
        let tempfile = env::temp_dir().join(basename);
        ttfont.call_method1("save", (tempfile.to_str().unwrap(),))?;
        let testable = Testable::new(&tempfile).expect("Couldn't create testable");

        // Run the check!
        let result = check
            .run(
                &fontspector_checkapi::TestableType::Single(&testable),
                &Context::default(),
                None,
            )
            .expect("Check failed");
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

pub fn run_python_test(module: &str, function: &str) {
    // Save cwd
    let cwd = env::current_dir().unwrap();
    setup_python();
    // Change to the fontbakery directory
    env::set_current_dir(fontbakery_directory()).unwrap();
    Python::with_gil(|py| {
        // First we import original fontbakery.codetesting.
        let codetesting =
            PyModule::import_bound(py, "fontbakery.codetesting").expect("Can't load codetesting");
        // Now replace CheckTester with our custom version
        let func: Bound<'_, PyCFunction> =
            wrap_pyfunction_bound!(checktester, py).expect("Failed to wrap checktester");
        codetesting
            .setattr("CheckTester", func)
            .expect("Failed to replace CheckTester");

        // Now run the original test
        let module = PyModule::import_bound(py, module).unwrap();
        let test = module
            .getattr(function)
            .expect("Failed to find test function");
        test.call0().unwrap_or_else(|e| {
            e.print_and_set_sys_last_vars(py);
            panic!("Test failed: {}", e);
        });
    });
    env::set_current_dir(cwd).unwrap();
}

static python_ready: Mutex<bool> = Mutex::new(false);

fn fontbakery_directory() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    let mut fb_dir = cargo_path.parent().unwrap().to_path_buf();
    fb_dir.push("fontbakery-bridge/fontbakery");
    fb_dir
}

fn setup_python() {
    let mut _guard = python_ready.lock().unwrap();
    if *_guard {
        return;
    }
    pyo3::prepare_freethreaded_python();
    let _res: PyResult<()> = Python::with_gil(|py| {
        let sys = py
            .import_bound("sys")?
            .getattr("path")
            .expect("Can't find path");
        let sys: Bound<'_, PyList> = sys.downcast_into()?;
        sys.insert(0, fontbakery_directory())?;
        Ok(())
    });
    *_guard = true;
}
