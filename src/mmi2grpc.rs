use super::Interaction;
use pyo3::{
    prelude::{PyErr, PyResult, Python},
    types::{PyBytes, PyModule, PyString},
};

pub struct Mmi2grpc {}

pub type Error = PyErr;

impl Mmi2grpc {
    pub fn new() -> Self {
        Python::with_gil(|py| -> Self {
            py.run("import sys; sys.path.append('mmi2grpc/')", None, None)
                .expect("Should not failed");
            Self {}
        })
    }

    pub fn reset(&self) -> Result<(), Error> {
        Python::with_gil(|py| -> PyResult<()> {
            PyModule::import(py, "mmi2grpc")?
                .getattr("reset")?
                .call((), None)?;
            Ok(())
        })
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
        })?;

        Ok(String::from("Ok"))
    }
}
