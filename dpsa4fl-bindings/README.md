
# Python bindings for dpsa4fl

**Warning: This project is work in progress and should not be used in production. The current implementation is a prototype.**

The dpsa4fl project aims at providing a mechanism for secure and differentially private aggregation
of gradients in federated machine learning. For more information see the [project overview](https://github.com/dpsa-project/overview).

## About this package
This package provides python bindings for the [dpsa4fl](https://github.com/dpsa-project/dpsa4fl) library.
The following functionality is provided:
 - **Controller api**: start a training session on the janus server, and collect aggregated gradients.
 - **Client api**: securely submit gradients to the janus server.
 
A modified [janus server setup](https://github.com/dpsa-project/dpsa4fl-testing-infrastructure) is required,
see the [example project](https://github.com/dpsa-project/dpsa4fl-example-project) for step-by-step instructions.

## Development notes
To release a new version of this package, you have to:
 1. Increment the version number in `Cargo.toml`.
 2. Push the current state to the `release` branch. Then github actions will do the rest.
    Alternatively, you can use [act](https://github.com/nektos/act) to run github actions locally.

Use [`maturin`](https://github.com/PyO3/maturin) to build the python package. To do so using the [`poetry`](https://python-poetry.org/) python package manager, do the following in the `dpsa4fl-bindings` folder:
```
poetry shell
maturin develop
```
