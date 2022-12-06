
use crate::core::PyControllerState;
use crate::core::PyControllerState_Mut;
use crate::core::PyMeasurement;
use std::ffi::CString;

use dpsa4fl::client::ClientStatePU;
use dpsa4fl::client::Fx;
use dpsa4fl::client::Measurement;
use dpsa4fl::client::RoundSettings;
use dpsa4fl::client::api__new_client_state;
use dpsa4fl::client::api__submit;
use dpsa4fl::controller::api__collect;
use dpsa4fl::controller::api__start_round;
use dpsa4fl::core::Locations;
use numpy::IxDyn;
use numpy::PyArray;
use numpy::PyArrayDyn;
use numpy::PyReadonlyArrayDyn;
use pyo3::with_embedded_python_interpreter;
use pyo3::{prelude::*, types::PyCapsule};
use dpsa4fl::{*, controller::{api__new_controller_state, ControllerState_Mut, ControllerState_Immut, api__create_session, ControllerState_Permanent}, core::{CommonState_Parametrization}};
use url::Url;
use anyhow::{anyhow, Result};
use tokio::runtime::Runtime;

use fixed::types::extra::{U15, U31, U63};
use fixed::{FixedI16, FixedI32, FixedI64};
use fixed_macro::fixed;

pub mod core;

/////////////////////////////////////////////////////////////////
// Param
fn get_common_state_parametrization() -> Result<CommonState_Parametrization>
{
    let res = CommonState_Parametrization {
        location: Locations {
            external_leader_main: Url::parse("http://127.0.0.1:9991")?,
            external_helper_main: Url::parse("http://127.0.0.1:9992")?,
            external_leader_tasks: Url::parse("http://127.0.0.1:9981")?,
            external_helper_tasks: Url::parse("http://127.0.0.1:9982")?,
            internal_leader: Url::parse("http://aggregator1:9991")?,
            internal_helper: Url::parse("http://aggregator2:9992")?,
        },
        gradient_len: 16,
    };
    Ok(res)
}

/////////////////////////////////////////////////////////////////
// Client api

#[pyclass]
struct PyClientState
{
    mstate: ClientStatePU,
}

#[pyfunction]
fn client_api__new_state() -> Result<PyClientState>
{
    let p = get_common_state_parametrization()?;
    let res = PyClientState {
        mstate: api__new_client_state(p)
    };
    Ok(res)
}

// fn run_on_client<'a, A, B: 'a, F>
//     (
//         client_state: Py<PyClientState>,
//         b: &'a B,
//         f: F,
//     )
//     -> Result<A>
//     where F: FnOnce(&'a mut ClientStatePU, &'a B) -> Result<A>,
// {
//     Python::with_gil(|py| {
//         let state_cell: &PyCell<PyClientState> = client_state.as_ref(py);
//         let mut state_ref_mut = state_cell.try_borrow_mut().map_err(|_| anyhow!("could not get mut ref"))?;
//         let state: &mut PyClientState = &mut *state_ref_mut;

//         // let istate : &ClientState_Immut = unsafe {state.istate.as_ref(py).reference()};
//         // let mut mstate : ClientState_Mut = state.mstate.clone().try_into()?;
//         // let mut mut_state: ControllerState = state.clone();
//         // execute async function in tokio runtime
//         let res = f(&mut state.mstate, &b)?;

//         Ok(res)
//     })
// }

#[pyfunction]
fn client_api__submit(client_state: Py<PyClientState>, task_id: String, data: PyReadonlyArrayDyn<f64>) -> Result<()>
{
    let round_settings = RoundSettings::new(task_id)?;

    Python::with_gil(|py| {
        let state_cell: &PyCell<PyClientState> = client_state.as_ref(py);
        let mut state_ref_mut = state_cell.try_borrow_mut().map_err(|_| anyhow!("could not get mut ref"))?;
        let state: &mut PyClientState = &mut *state_ref_mut;

        let zero: Fx = fixed!(0: I1F31);
        let data: Measurement = vec![zero; state.mstate.get_parametrization().gradient_len];

        let res = Runtime::new().unwrap().block_on(api__submit(&mut state.mstate, round_settings, &data))?;

        Ok(res)
    })
}

/////////////////////////////////////////////////////////////////
// Controller api

#[pyfunction]
fn controller_api__new_state() -> Result<PyControllerState>
{
    let p = get_common_state_parametrization()?;
    let istate = api__new_controller_state(p);
    let istate : Py<PyCapsule> = Python::with_gil(|py| {
        let capsule = PyCapsule::new(py, istate, None);
        capsule.map(|c| c.into())
    }).unwrap();

    let mstate = PyControllerState_Mut {
        training_session_id: None,
        task_id: None
    };

    let res = PyControllerState {
        mstate,
        istate,
    };

    Ok(res)
}


fn run_on_controller<A>
    (
        controller_state: Py<PyControllerState>,
        f: fn(&ControllerState_Immut, &mut ControllerState_Mut) -> Result<A>,
    )
    -> Result<A>
{
    Python::with_gil(|py| {
        let state_cell: &PyCell<PyControllerState> = controller_state.as_ref(py);
        let mut state_ref_mut = state_cell.try_borrow_mut().map_err(|_| anyhow!("could not get mut ref"))?;
        let state: &mut PyControllerState = &mut *state_ref_mut;

        let istate : &ControllerState_Immut = unsafe {state.istate.as_ref(py).reference()};
        let mut mstate : ControllerState_Mut = state.mstate.clone().try_into()?;
        // let mut mut_state: ControllerState = state.clone();
        // execute async function in tokio runtime
        let res = f(istate, &mut mstate)?;

        // write result into state
        state.mstate = mstate.into();

        Ok(res)
    })
}

#[pyfunction]
fn controller_api__create_session(controller_state: Py<PyControllerState>) -> Result<u16>
{
    run_on_controller(
        controller_state,
        |i,m| Runtime::new().unwrap().block_on(api__create_session(i, m))
    )
}

#[pyfunction]
fn controller_api__start_round(controller_state: Py<PyControllerState>) -> Result<String>
{
    run_on_controller(
        controller_state,
        |i,m| Runtime::new().unwrap().block_on(api__start_round(i, m))
    )
}

#[pyfunction]
fn controller_api__collect(controller_state: Py<PyControllerState>) -> Result<String>
{
    let res = run_on_controller(
        controller_state,
        |i,m| Runtime::new().unwrap().block_on(api__collect(i, m))
    )?;

    Ok(format!("Result: {:?}", res))
}



/// A Python module implemented in Rust.
#[pymodule]
fn dpsa4fl_bindings(_py: Python, m: &PyModule) -> PyResult<()>
{
    // add class
    m.add_class::<PyControllerState>()?;
    m.add_class::<PyControllerState_Mut>()?;

    // add functions
    //--- controller api ---
    m.add_function(wrap_pyfunction!(controller_api__new_state, m)?)?;
    m.add_function(wrap_pyfunction!(controller_api__create_session, m)?)?;
    m.add_function(wrap_pyfunction!(controller_api__start_round, m)?)?;
    m.add_function(wrap_pyfunction!(controller_api__collect, m)?)?;
    //--- client api ---
    m.add_function(wrap_pyfunction!(client_api__new_state, m)?)?;
    m.add_function(wrap_pyfunction!(client_api__submit, m)?)?;

    Ok(())
}

