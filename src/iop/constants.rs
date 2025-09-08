//! Optical constants and coefficient data
//!
//! This module contains pre-defined optical coefficients for water and phytoplankton
//! at various wavelengths used in bio-optical calculations.

use std::collections::BTreeMap;
use std::sync::LazyLock;

/// Water absorption coefficients at different wavelengths (nm)
/// Values represent absorption coefficient in m^-1
/// Data from Pope and Fry (1997) and other standard oceanographic references
/// https://oceancolor.gsfc.nasa.gov/docs/rsr/water_coef.txt
pub static AW_ALL: LazyLock<BTreeMap<u32, f64>> = LazyLock::new(|| {
    BTreeMap::from([
        (410, 0.00473),
        (412, 0.00455056),
        (443, 0.00706914),
        (469, 0.0104326),
        (486, 0.0139217),
        (488, 0.0145167),
        (490, 0.015),
        (510, 0.0325),
        (531, 0.0439153),
        (547, 0.0531686),
        (551, 0.0577925),
        (555, 0.0596),
        (645, 0.325),
        (667, 0.434888),
        (670, 0.439),
        (671, 0.442831),
        (678, 0.462323),
    ])
});

/// Water backscattering coefficients at different wavelengths (nm)
/// Values represent backscattering coefficient in m^-1
/// Data from Zhang et al. (2009) and other standard oceanographic references
pub static BBW_ALL: LazyLock<BTreeMap<u32, f64>> = LazyLock::new(|| {
    BTreeMap::from([
        (410, 0.00339515),
        (412, 0.003325),
        (443, 0.002436175),
        (469, 0.001908315),
        (486, 0.0016387),
        (488, 0.001610175),
        (490, 0.001582255),
        (510, 0.001333585),
        (531, 0.001122495),
        (547, 0.000988925),
        (551, 0.000958665),
        (555, 0.000929535),
        (645, 0.00049015),
        (667, 0.000425025),
        (670, 0.000416998),
        (671, 0.000414364),
        (678, 0.000396492),
    ])
});

/// Phytoplankton specific absorption coefficients at different wavelengths (nm)
/// Values represent specific absorption coefficient in m^2/mg
/// Data from Bricaud et al. (1995, 1998) and other phytoplankton optical studies
pub static APHSTAR_ALL: LazyLock<BTreeMap<u32, f64>> = LazyLock::new(|| {
    BTreeMap::from([
        (410, 0.054343207),
        (412, 0.055765253),
        (443, 0.063251586),
        (469, 0.051276462),
        (486, 0.041649554),
        (488, 0.040647623),
        (490, 0.039546143),
        (510, 0.025104817),
        (531, 0.015745358),
        (547, 0.011477324),
        (551, 0.010425453),
        (555, 0.009381989),
        (645, 0.008966522),
        (667, 0.019877564),
        (670, 0.022861409),
        (671, 0.023645549),
        (678, 0.024389358),
    ])
});

/// QAA reference wavelength (nm)
pub const LAMBDA_0: u32 = 555;

/// Water scattering coefficient at 555nm (m^-1)
/// From Zhang et al. (2009) and standard ocean optics literature
pub const BW_555: f64 = 0.0038;

/// Spectral slope for particle backscattering
/// Standard QAA algorithm parameter from Lee et al. (2002)
pub const ETA: f64 = 0.5;

/// Spectral slope for CDOM absorption  
/// Standard QAA algorithm parameter from Lee et al. (2002)
pub const S: f64 = 0.015;

pub const G0: f64 = 0.08945;
pub const G1: f64 = 0.1247;

/// QAA step thresholds
/// Algorithm decision thresholds from Lee et al. (2002) QAA implementation
pub const RRS_670_THRESHOLD: f64 = 0.0015;

/// Default coefficients for SeaWiFS
/// https://oceancolor.gsfc.nasa.gov/docs/ocssw/qaa_8c_source.html
pub const C1: f64 = -1.146;
pub const C2: f64 = -1.366;
pub const C3: f64 = -0.469;
