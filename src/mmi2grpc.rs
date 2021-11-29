use std::fmt;

use pyo3::{
    prelude::{PyErr, PyResult, Python},
    types::{PyBytes, PyModule, PyString},
};

use super::Interaction;

pub struct Mmi2grpc;

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

impl Mmi2grpc {
    pub fn new() -> Self {
        Python::with_gil(|py| -> Self {
            py.run("import sys; sys.path.append('mmi2grpc/')", None, None)
                .expect("Should not fail");
            Self
        })
    }

    pub fn reset(&self) -> Result<(), Error> {
        Python::with_gil(|py| -> PyResult<()> {
            PyModule::import(py, "mmi2grpc")?
                .getattr("reset")?
                .call((), None)?;
            Ok(())
        })
        .map_err(Error)
    }

    pub fn read_local_address(&self) -> Result<Vec<u8>, Error> {
        Python::with_gil(|py| -> PyResult<Vec<u8>> {
            Ok(PyModule::import(py, "mmi2grpc")?
                .getattr("read_local_address")?
                .call((), None)?
                .cast_as::<PyBytes>()?
                .as_bytes()
                .to_vec())
        })
        .map_err(Error)
    }

    pub fn interact(&self, interaction: Interaction<'_>) -> Result<String, Error> {
        Python::with_gil(|py| -> PyResult<()> {
            let interaction_id = PyString::new(py, interaction.id);
            let profile = PyString::new(py, interaction.profile);
            let pts_addr = PyBytes::new(py, &*interaction.pts_addr);
            PyModule::import(py, "mmi2grpc")?.getattr("run")?.call(
                (
                    profile,
                    interaction_id,
                    interaction.test.to_string(),
                    pts_addr,
                ),
                None,
            )?;
            Ok(())
        })
        .map_err(Error)?;

        Ok(String::from("Ok"))
    }
}
