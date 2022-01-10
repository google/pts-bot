use std::fmt;

use pyo3::{
    types::{PyBytes, PyModule, PyString},
    Py, PyErr, PyResult, Python,
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

            Ok(stringio.call_method0("getvalue")?.extract()?)
        });
        write!(f, "{}", value.unwrap())
    }
}

impl std::error::Error for Error {}

pub struct PythonIUT(Py<PyModule>);

impl PythonIUT {
    pub fn new(name: &str) -> Result<Self, Error> {
        Python::with_gil(|py| -> PyResult<Self> { Ok(Self(PyModule::import(py, name)?.into())) })
            .map_err(Error)
    }

    pub fn reset(&self) -> Result<(), Error> {
        Python::with_gil(|py| -> PyResult<()> {
            self.0.as_ref(py).getattr("reset")?.call0()?;
            Ok(())
        })
        .map_err(Error)
    }

    pub fn read_local_address(&self) -> Result<Vec<u8>, Error> {
        Python::with_gil(|py| -> PyResult<Vec<u8>> {
            Ok(self
                .0
                .as_ref(py)
                .getattr("read_local_address")?
                .call0()?
                .cast_as::<PyBytes>()?
                .as_bytes()
                .to_vec())
        })
        .map_err(Error)
    }

    pub fn interact(&self, interaction: Interaction) -> Result<String, Error> {
        Python::with_gil(|py| -> PyResult<String> {
            let (addr, _style, id, profile, test, description) = interaction.explode();
            Ok(self
                .0
                .as_ref(py)
                .getattr("run")?
                .call((profile, id, test, description, &*addr as &[u8]), None)?
                .cast_as::<PyString>()?
                .to_string_lossy()
                .into_owned())
        })
        .map_err(Error)
    }
}

pub fn wait_signal<T>() -> Result<T, Error> {
    Python::with_gil(|py| -> PyResult<T> {
        let signal_pause = PyModule::import(py, "signal")?.getattr("pause")?;

        loop {
            signal_pause.call0()?;
        }
    })
    .map_err(Error)
}
