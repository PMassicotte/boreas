# Primary Production in the Arctic

[![Cargo Build & Test](https://github.com/PMassicotte/primprod/actions/workflows/rust.yml/badge.svg)](https://github.com/PMassicotte/primprod/actions/workflows/rust.yml)

Based on my experience, many of the current models for estimating aquatic primary production in the Arctic are challenging to use, difficult to extend, and often lack thorough documentation. I'm starting to explore the idea of developing a new Open Source model, built from the ground up with a modern tool stack, to provide a more accessible and adaptable solution. This model would aim to support the scientific community in better understanding and managing Arctic ecosystems.

## Ideas

- **Open Source**: The model should be freely available to all, and the code should be well-documented and easy to understand.

- **Modular Design**: The model should be built in a modular way, allowing users to easily extend and modify it.

- **Highly configurable**: The model should be highly configurable, allowing users to customize it to their specific needs.
  - **Spatial resolution**: The model should support different spatial resolutions, from global to local scales.
  - **Temporal resolution**: The model should support different temporal resolutions, from daily to annual scales.
  - **Input data**: The model should support different types of input data, such as satellite imagery, in situ measurements, and model outputs.

- **Low level programming**: The model should be written in a low-level programming language like C or Rust to ensure high performance.

- **Low deoendency**: The model should have as few dependencies as possible to make it easy to install and run.

- **h3**: The model should use the h3 library for spatial indexing and analysis.

## Using a logger

- https://medium.com/nerd-for-tech/logging-in-rust-e529c241f92e
  - https://crates.io/crates/env_logger
  - https://crates.io/crates/log

- https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html

## Implementation

### Spatial data

- How to deal with all the different input data formats?
  - NetCDF
  - GeoTIFF
  - CSV
  - JSON
  - Shapefile

- How to handle different spatial resolutions?

- How to handle different coordinate reference systems?

- How to handle different spatial indexing methods?

- Should it only support a single format, or should it be able to handle multiple formats?
  - Maybe cloud optimized GeoTIFFs? This would allow us to use the same data in different resolutions and projections and easily access it from the cloud.
  - Maybe we could use GDAL to handle all the different formats?

## Modules

- iop
- aop
- phytoplankton physiological parameters (PvsE, ...)
- light attenuation
- primary production
