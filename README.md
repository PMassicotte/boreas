<p align="center">
  <img src="logo.svg" alt="Boreas Logo", width="300"/>
</p>

# Boreas - Arctic Primary Production Model

[![Cargo Build & Test](https://github.com/PMassicotte/boreas/actions/workflows/rust.yml/badge.svg)](https://github.com/PMassicotte/boreas/actions/workflows/rust.yml) ![Experimental](https://img.shields.io/badge/status-experimental-orange)

Based on my experience, many of the current models for estimating aquatic primary production in the Arctic are challenging to use, difficult to extend, and often lack thorough documentation. I'm starting to explore the idea of developing a new Open Source model, built from the ground up with a modern tool stack, to provide a more accessible and adaptable solution. This model would aim to support the scientific community in better understanding and managing Arctic ecosystems.

## How It Works

```mermaid
graph TD
    A[JSON Config Loading<br/>simple_config.json] --> B[Date Series Generation<br/>config/timestep.rs]
    B --> C[File Pattern Discovery<br/>BatchProcessor::create_datasets]
    C --> D[GDAL Dataset Loading<br/>oceanographic_model/processor.rs]
    D --> E[Spatial Region Processing<br/>Bounding Box Clipping]
    E --> F[Pixel-Level Processing<br/>oceanographic_model/pixel.rs]
    F --> G[VGPM Algorithm<br/>Pbopt × Chl × Zeu calculation]
    G --> H[GeoTIFF Output<br/>pp_YYYYMMDD.tif]
    
    I[Input Rasters] --> D
    I --> J[Chlorophyll-a<br/>mg/m³]
    I --> K[Sea Surface Temp<br/>°C]
    I --> L[Kd_490<br/>m⁻¹]
    J --> F
    K --> F
    L --> F

    style A fill:#ffffff,color:#000,stroke:#333
    style B fill:#ffffff,color:#000,stroke:#333
    style C fill:#ffffff,color:#000,stroke:#333
    style D fill:#ffffff,color:#000,stroke:#333
    style E fill:#ffffff,color:#000,stroke:#333
    style F fill:#ffffff,color:#000,stroke:#333
    style G fill:#ffffff,color:#000,stroke:#333
    style H fill:#ffffff,color:#000,stroke:#333
    style I fill:#ffffff,color:#000,stroke:#333
    style J fill:#ffffff,color:#000,stroke:#333
    style K fill:#ffffff,color:#000,stroke:#333
    style L fill:#ffffff,color:#000,stroke:#333
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
