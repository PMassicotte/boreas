<p align="center">
  <img src="logo.svg" alt="Boreas Logo", width="300"/>
</p>

# Boreas - Arctic Primary Production Model

[![Cargo Build & Test](https://github.com/PMassicotte/boreas/actions/workflows/rust.yml/badge.svg)](https://github.com/PMassicotte/boreas/actions/workflows/rust.yml) ![Experimental](https://img.shields.io/badge/status-experimental-orange)

Based on my experience, many of the current models for estimating aquatic primary production in the Arctic are challenging to use, difficult to extend, and often lack thorough documentation. I'm starting to explore the idea of developing a new Open Source model, built from the ground up with a modern tool stack, to provide a more accessible and adaptable solution. This model would aim to support the scientific community in better understanding and managing Arctic ecosystems.

## How It Works

```mermaid
graph TD
    A[Configuration File] --> B[Find Satellite Data Files]
    B --> C[Load MODIS Rasters<br/>Chlorophyll, SST, Kd_490]
    C --> D[Apply VGPM Algorithm]
    D --> E[Primary Production Output<br/>mg C m−2 d−1]

    style A fill:#e1f5fe
    style C fill:#f3e5f5
    style D fill:#fff3e0
    style E fill:#e8f5e8
```

## Reference Implementations

### NASA OCSSW QAA Reference

The `reference/nasa-ocssw/` directory contains a C reference implementation of the Quasi-Analytical Algorithm (QAA) v6, directly based on NASA's Ocean Color Science Software (OCSSW).

**Purpose**: Validation and comparison baseline for the Rust QAA implementation in `src/iop/qaa.rs`

**Usage**:

```bash
# Build and run the reference implementation
make -C reference/nasa-ocssw run

# View detailed documentation
cat reference/nasa-ocssw/README.md
```

**Source**: https://oceancolor.gsfc.nasa.gov/docs/ocssw/qaa_8c_source.html

## Acknowledgments

The atmospheric LUT data used in this model is provided by Simon Belanger (UQAR) from 2011.
