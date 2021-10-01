use super::Interaction;
use pyo3::{
    prelude::{Python, PyResult},
    types::{PyBytes, PyModule, PyString},
};

pub fn run(interaction: Interaction<'_>) -> Result<String, Box<dyn std::error::Error>> {
    Python::with_gil(|py| -> PyResult<()> {
        py.run("import sys; sys.path.append('mmi2grpc/')", None, None)?;
        let interact_module = PyModule::import(py, "interact")?;
        let profile = PyString::new(py, interaction.profile);
        let interaction_id = PyString::new(py, interaction.id);
        let pts_addr = PyBytes::new(py, &format!("{}", interaction.pts_addr).into_bytes());
        interact_module
            .getattr("run")?
            .call((profile, interaction_id, pts_addr), None)?;
        Ok(())
    })?;
    Ok(String::from("Ok"))
}