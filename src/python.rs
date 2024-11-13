use std::fmt;

use pyo3::{
    types::{IntoPyDict, PyBytes, PyDict, PyModule, PyString},
    PyErr, PyObject, PyResult, Python,
};

use super::Interaction;

#[derive(Debug)]
pub struct Error(PyErr);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = Python::with_gil(|py| -> PyResult<String> {
            let io = PyModule::import(py, "io")?;
            let stringio = io.getattr("StringIO")?.call0()?;
            let sys = PyModule::import(py, "sys")?;
            let old_stderr = sys.getattr("stderr")?;
            sys.setattr("stderr", stringio)?;
            self.0.print(py);
            sys.setattr("stderr", old_stderr)?;

            stringio.call_method0("getvalue")?.extract()
        });
        write!(f, "{}", value.unwrap())
    }
}

impl std::error::Error for Error {}

pub struct PythonIUT(PyObject);

/// Bind to an IUT python object with the following template:
///
/// class IUT:
///    def __init__(self, test: str, args: List[str], **kwargs):
///        """Initialize the instance manager."""
///        pass
///
///    def __enter__(self):
///        """Start the instance subprocess."""
///        pass
///
///    def __exit__(self):
///        """Kill the instance subprocess."""
///        pass
///
///    @property
///    def address(self) -> bytes:
///        pass
///
///    def interact(self,
///                 pts_address: bytes,
///                 profile: str,
///                 test: str,
///                 interaction: str,
///                 description: str,
///                 style: str,
///                 **kwargs) -> str:
///        """Send an interaction to the instance subprocess and wait for
///           the result."""
///        pass
///
impl PythonIUT {
    pub fn new(name: &str, args: &Vec<String>, test: &str) -> Result<Self, Error> {
        Python::with_gil(|py| -> PyResult<Self> {
            let kwargs = PyDict::new(py);
            kwargs.set_item("test", test)?;
            kwargs.set_item("args", args)?;
            PyModule::import(py, name)?
                .getattr("IUT")?
                .call((), Some(kwargs))
                .map(|obj| Self(obj.into()))
        })
        .map_err(Error)
    }

    pub fn enter(&self) -> Result<(), Error> {
        Python::with_gil(|py| -> PyResult<()> {
            let obj = self.0.as_ref(py);
            obj.call_method0("__enter__")?;
            Ok(())
        })
        .map_err(Error)
    }

    pub fn exit(&self) -> Result<(), Error> {
        Python::with_gil(|py| -> PyResult<()> {
            let obj = self.0.as_ref(py);
            obj.call_method0("__exit__")?;
            Ok(())
        })
        .map_err(Error)
    }

    pub fn address(&self) -> Result<Vec<u8>, Error> {
        Python::with_gil(|py| -> PyResult<Vec<u8>> {
            let obj = self.0.as_ref(py);
            Ok(obj
                .getattr("address")?
                .cast_as::<PyBytes>()?
                .as_bytes()
                .to_vec())
        })
        .map_err(Error)
    }

    pub fn interact(&self, interaction: Interaction) -> Result<String, Error> {
        Python::with_gil(|py| -> PyResult<String> {
            let (addr, style, id, profile, test, description) = interaction.explode();
            let style = format!("{:?}", style);
            let obj = self.0.as_ref(py);
            let args = ();
            let kwargs = [
                ("profile", profile),
                ("test", test),
                ("interaction", id),
                ("description", description),
                ("style", &style),
            ]
            .into_py_dict(py);
            kwargs.set_item("pts_address", PyBytes::new(py, &*addr))?;
            Ok(obj
                .call_method("interact", args, Some(kwargs))?
                .cast_as::<PyString>()?
                .to_string_lossy()
                .into_owned())
        })
        .map_err(Error)
    }
}

impl Drop for PythonIUT {
    fn drop(&mut self) {
        let _ = self.exit();
    }
}
