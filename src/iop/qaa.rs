//! Quasi-Analytical Algorithm (QAA v6) for Ocean Color Remote Sensing
//!
//! This module provides a Rust implementation of the Quasi-Analytical Algorithm (QAA) version 6,
//! closely following the NASA Ocean Color Science Software (OCSSW) reference implementation.
//!
//! ## NASA OCSSW Reference Implementation
//!
//! This implementation is based on the official NASA OCSSW QAA algorithm source code:
//! - **Reference URL**: <https://oceancolor.gsfc.nasa.gov/docs/ocssw/qaa_8c_source.html>
//! - **Algorithm Version**: QAA v6
//! - **NASA Coefficients**: Uses exact NASA OCSSW coefficients and constants
//! - **Wavelengths**: Standard NASA wavelengths [410, 443, 490, 555, 670] nm
//!
//! ## Algorithm Overview
//!
//! The QAA algorithm derives inherent optical properties (IOPs) from remote sensing
//! reflectance measurements. It follows a 6-step analytical approach:
//!
//! 1. **Step 0**: Convert remote sensing reflectance (Rrs) to below-water reflectance
//! 2. **Step 1**: Calculate u parameter (related to backscattering ratio)
//! 3. **Step 2**: Determine reference absorption coefficient at 555nm
//! 4. **Step 3**: Calculate reference particulate backscattering coefficient
//! 5. **Step 4**: Derive spectral slope for particulate backscattering
//! 6. **Step 5**: Calculate total backscattering and absorption coefficients
//! 7. **Step 6**: Decompose absorption into phytoplankton and CDOM+detrital components
//!
//! ## NASA Compliance
//!
//! This implementation maintains strict compliance with NASA OCSSW:
//! - **Constants**: G0=0.089, G1=0.125 (exact NASA values)
//! - **Coefficients**: acoefs=[-1.146, -1.366, -0.469] for SeaWiFS/MODIS
//! - **Rrs Conversion**: rrs = Rrs / (0.52 + 1.7 * Rrs)
//! - **Reference Wavelength**: 555nm (primary reference as per NASA)
//! - **Quality Flagging**: Bitfield flags matching NASA OCSSW convention
//!
//! ## References
//!
//! - Lee, Z., Carder, K. L., & Arnone, R. A. (2002). Deriving inherent optical properties
//!   from water color: a multiband quasi-analytical algorithm for optically deep waters.
//!   *Applied Optics*, 41(27), 5755-5772.
//! - Lee, Z., et al. (2009). Euphotic zone depth: Its derivation and implication to
//!   ocean-color remote sensing. *Journal of Geophysical Research*, 114, C01009.
//! - NASA Ocean Color Science Software (OCSSW) QAA implementation
//!
//! ## Usage Example
//!
//! ```rust
//! use std::collections::BTreeMap;
//! use boreas::iop::qaa_v6;
//! use boreas::sat_bands::Satellites;
//!
//! let rrs = BTreeMap::from([
//!     (410, 0.001974),
//!     (443, 0.002570),
//!     (490, 0.002974),
//!     (555, 0.001670),
//!     (670, 0.000324),
//! ]);
//!
//! let result = qaa_v6(&rrs, Satellites::Modis);
//! println!("Chlorophyll-a: {:.3} mg/m3", result.chla);
//! ```

use crate::iop::constants;
use crate::sat_bands::{SatBands, Satellites};
use std::collections::BTreeMap;

/// QAA algorithm results
#[derive(Debug)]
pub struct QaaResult {
    pub wavelengths: Vec<u32>, // Wavelengths [nm]
    pub rrs: Vec<f64>,         // Below-water reflectance [sr^-1]
    pub u: Vec<f64>,           // U-ratio [dimensionless]
    pub a: Vec<f64>,           // Total absorption [m^-1]
    pub aph: Vec<f64>,         // Phytoplankton absorption [m^-1]
    pub acdom: Vec<f64>,       // CDOM (detrital+dissolved) absorption [m^-1]
    pub bb: Vec<f64>,          // Total backscattering [m^-1]
    pub bbp: Vec<f64>,         // Particulate backscattering [m^-1]
    pub flags: u8,             // Quality flags [bitfield]
    pub chla: f64,             // Chla [mg/m^3]
    pub version: String,       // Algorithm version (e.g., "QAA v6")
    pub reference_wl: u32,     // Reference wavelength used [nm]
    pub spectral_slope_y: f64, // Spectral slope Y for bbp
    pub spectral_slope_s: f64, // Spectral slope S for acdom
    pub aph_ratio_443: f64,    // aph/a ratio at 443nm for quality assessment
}

enum QAAMessage {
    InvalidData,
    NegativeBackscattering,
    DecompositionError,
    AphCorrectionApplied,
    NegativeAphValues,
    ChlorophyllCalculationError,
    AphRatioForcedMax,
    BackscatteringLessThanWater,
}

impl QAAMessage {
    fn as_str(&self) -> &'static str {
        match self {
            QAAMessage::InvalidData => "Invalid data for log calculation (negative Rrs ratios)",
            QAAMessage::NegativeBackscattering => "Negative particulate backscattering detected",
            QAAMessage::DecompositionError => "Absorption decomposition error (division by zero)",
            QAAMessage::AphCorrectionApplied => "aph/a ratio correction applied at 443nm",
            QAAMessage::NegativeAphValues => "Negative phytoplankton absorption values corrected",
            QAAMessage::ChlorophyllCalculationError => "Chlorophyll calculation error",
            QAAMessage::AphRatioForcedMax => "aph/a ratio forced to maximum (0.6)",
            QAAMessage::BackscatteringLessThanWater => {
                "Backscattering less than water backscattering"
            }
        }
    }
}

impl QaaResult {
    pub fn get_messages(&self) -> Vec<String> {
        let mut messages = Vec::new();

        if self.flags & 0x01 != 0 {
            messages.push(QAAMessage::InvalidData.as_str().to_string());
        }
        if self.flags & 0x02 != 0 {
            messages.push(QAAMessage::NegativeBackscattering.as_str().to_string());
        }
        if self.flags & 0x04 != 0 {
            messages.push(QAAMessage::DecompositionError.as_str().to_string());
        }
        if self.flags & 0x08 != 0 {
            messages.push(QAAMessage::AphCorrectionApplied.as_str().to_string());
        }
        if self.flags & 0x10 != 0 {
            messages.push(QAAMessage::NegativeAphValues.as_str().to_string());
        }
        if self.flags & 0x20 != 0 {
            messages.push(QAAMessage::ChlorophyllCalculationError.as_str().to_string());
        }
        if self.flags & 0x40 != 0 {
            messages.push(QAAMessage::AphRatioForcedMax.as_str().to_string());
        }
        if self.flags & 0x80 != 0 {
            messages.push(QAAMessage::BackscatteringLessThanWater.as_str().to_string());
        }

        messages
    }
}

pub fn subset_optical_data(wavelengths: &[u32], data: &BTreeMap<u32, f64>) -> BTreeMap<u32, f64> {
    wavelengths
        .iter()
        .map(|&lambda| {
            let closest_wl = data
                .keys()
                .min_by_key(|&&wl| (wl as i32 - lambda as i32).abs())
                .unwrap();
            (lambda, *data.get(closest_wl).unwrap())
        })
        .collect()
}

fn calculate_acdom_absorption(
    wavelengths: &[u32],
    ag440: f64,
    spectral_slope: f64,
    reference_wl: u32,
) -> BTreeMap<u32, f64> {
    wavelengths
        .iter()
        .map(|&wl| {
            let acdom_val = ag440 * (spectral_slope * (reference_wl as f64 - wl as f64)).exp();
            (wl, acdom_val)
        })
        .collect()
}

fn calculate_phytoplankton_absorption(
    wavelengths: &[u32],
    total_absorption: &BTreeMap<u32, f64>,
    acdom_absorption: &BTreeMap<u32, f64>,
    water_absorption: &BTreeMap<u32, f64>,
) -> BTreeMap<u32, f64> {
    wavelengths
        .iter()
        .map(|&wl| {
            let aph_val = total_absorption.get(&wl).unwrap()
                - acdom_absorption.get(&wl).unwrap()
                - water_absorption.get(&wl).unwrap();
            (wl, aph_val)
        })
        .collect()
}

pub fn qaa_v6(rrs: &BTreeMap<u32, f64>, satellite: Satellites) -> QaaResult {
    // Initialize quality flags
    let mut flags = 0u8;

    // NASA QAA v6 target wavelengths (nm)
    let nasa_target_wavelengths = [410, 443, 490, 555, 670];

    // Create SatBands for wavelength mapping
    let sat_bands = SatBands::new(satellite);

    // Map NASA target wavelengths to closest available satellite bands
    let wavelengths: Vec<u32> = nasa_target_wavelengths
        .iter()
        .map(|&target| sat_bands.closest_band(target))
        .collect();

    // Subset aw, bbw, and aphstar to the mapped wavelengths
    let aw = subset_optical_data(&wavelengths, &constants::AW_ALL);
    let bbw = subset_optical_data(&wavelengths, &constants::BBW_ALL);
    let aphstar = subset_optical_data(&wavelengths, &constants::APHSTAR_ALL);

    let mut rrs = subset_optical_data(&wavelengths, rrs);

    // Convert rrs to below sea level (NASA formulation)
    rrs.iter_mut()
        .for_each(|(_k, v)| *v = *v / (0.52 + (1.7 * *v)));

    // Step 1: Calculate the diffusion probabilities at each wavelengths
    let u: BTreeMap<u32, f64> = rrs
        .iter()
        .map(|(k, v)| {
            let u = ((constants::G0.powi(2) + 4.0 * constants::G1 * v).sqrt() - constants::G0)
                / (2.0 * constants::G1);

            (*k, u)
        })
        .collect();

    // Step 2: Determine reference wavelength and absorption coefficient (NASA OCSSW approach)
    // Map NASA target wavelengths to actual satellite bands
    let red_wl = sat_bands.closest_band(670);
    let green_wl = sat_bands.closest_band(555); // reference wavelength
    let blue_wl = sat_bands.closest_band(490);
    let cyan_wl = sat_bands.closest_band(443);
    let violet_wl = sat_bands.closest_band(410); // NASA uses 410, not 412

    // NASA QAA v6 uses 555nm as primary reference wavelength
    let wvlref = green_wl;
    let rrs_443 = rrs.get(&cyan_wl).unwrap();
    let rrs_490 = rrs.get(&blue_wl).unwrap();
    let rrs_555 = rrs.get(&green_wl).unwrap();
    let rrs_670 = rrs.get(&red_wl).unwrap();

    // NASA OCSSW coefficients for SeaWiFS
    let acoefs = [-1.146, -1.366, -0.469];

    // Calculate ratio for absorption estimation
    let numer = rrs_443 + rrs_490;
    let denom = rrs_555 + 5.0 * (rrs_670 * rrs_670) / rrs_490;

    // Bounds check for log calculation
    if denom <= 0.0 || numer <= 0.0 {
        flags |= 0x01; // Set invalid data flag
    }

    let aux = (numer / denom).max(1e-10).log10();
    let rho = acoefs[0] + acoefs[1] * aux + acoefs[2] * aux.powi(2);
    let aref = aw.get(&wvlref).unwrap() + 10.0_f64.powf(rho);

    // Step 3: Calculate reference backscattering
    let u_ref = u.get(&wvlref).unwrap();
    let bbpref = u_ref * aref / (1.0 - u_ref) - bbw.get(&wvlref).unwrap();

    // Check for negative bbp
    if bbpref < 0.0 {
        flags |= 0x02; // Set negative bbp flag
    }

    // Step 4: Calculate spectral slope Y (NASA OCSSW formulation)
    let rat = rrs_443 / rrs_555;
    let y = 2.0 * (1.0 - 1.2 * (-0.9 * rat).exp());

    // Bounds check for Y
    let y = y.clamp(0.0, 3.0);

    // Step 5: Calculate total backscattering bb
    let bb: BTreeMap<u32, f64> = wavelengths
        .iter()
        .map(|&wl| {
            let bb_val = bbpref * ((wvlref as f64) / (wl as f64)).powf(y) + bbw.get(&wl).unwrap();
            (wl, bb_val)
        })
        .collect();

    // Step 6: Calculate total absorption a
    let a: BTreeMap<u32, f64> = wavelengths
        .iter()
        .map(|&wl| {
            let u_val = u.get(&wl).unwrap();
            let bb_val = bb.get(&wl).unwrap();
            let a_val = ((1.0 - u_val) * bb_val) / u_val;
            (wl, a_val)
        })
        .collect();

    // Step 7: Calculate symbol coefficient (NASA formulation)
    let symbol = 0.74 + 0.2 / (0.8 + rat);

    // Step 8: Calculate spectral slope Sr (NASA formulation)
    let sr = constants::S + 0.002 / (0.6 + rat);
    let zeta = (sr * (cyan_wl as f64 - violet_wl as f64)).exp(); // Use actual mapped wavelengths

    // Step 9: Calculate ag at 443nm and decompose absorption
    let denom = zeta - symbol;

    // Check for division by zero
    if denom.abs() < 1e-10 {
        flags |= 0x04; // Set decomposition error flag
    }

    let a_410 = a.get(&violet_wl).unwrap();
    let a_443 = a.get(&cyan_wl).unwrap();
    let aw_410 = aw.get(&violet_wl).unwrap();
    let aw_443 = aw.get(&cyan_wl).unwrap();

    let dif1 = a_410 - symbol * a_443;
    let dif2 = aw_410 - symbol * aw_443;
    let acdom443 = (dif1 - dif2) / denom.max(1e-10); // Use 443nm reference

    // Calculate initial adg and aph using helper functions
    let initial_adg = calculate_acdom_absorption(&wavelengths, acdom443, sr, cyan_wl);
    let initial_aph = calculate_phytoplankton_absorption(&wavelengths, &a, &initial_adg, &aw);

    // Check and correct aph at 443nm (NASA bounds)
    let mut x1 = initial_aph.get(&cyan_wl).unwrap() / a_443;

    // NASA QAA v6: aph proportion should be between 0.15 and 0.6
    if !(0.15..=0.6).contains(&x1) || !x1.is_finite() {
        x1 = -0.8 + 1.4 * (a_443 - aw_443) / (a_410 - aw_410);
        flags |= 0x08; // Set aph correction flag
    }

    // Clamp to NASA bounds
    x1 = x1.clamp(0.15, 0.6);

    // Recalculate acdom443 based on corrected aph at 443nm
    let corrected_acdom443 = a_443 - (a_443 * x1) - aw_443;

    // Final calculations with corrected acdom443
    let mut acdom = calculate_acdom_absorption(&wavelengths, corrected_acdom443, sr, cyan_wl);
    let mut aph = calculate_phytoplankton_absorption(&wavelengths, &a, &acdom, &aw);

    // Handle negative aph values (NASA QAA v6 approach)
    for (&wl, aph_val) in aph.iter_mut() {
        if *aph_val < 0.0 {
            flags |= 0x10; // Set negative aph flag

            let aw_val = *aw.get(&wl).unwrap();
            let a_val = *a.get(&wl).unwrap();

            // NASA approach: set minimum values and flag
            let min_aph = 0.001;
            *aph_val = min_aph;

            let corrected_acdom = a_val - min_aph - aw_val;
            if let Some(acdom_val) = acdom.get_mut(&wl) {
                *acdom_val = corrected_acdom.max(0.0);
            }
        }
    }

    // Calculate chlorophyll concentration (NASA method)
    let aph_443_val = *aph.get(&cyan_wl).unwrap();
    let aphstar_443_val = *aphstar.get(&cyan_wl).unwrap();

    let chla = if aphstar_443_val > 0.0 && aph_443_val.is_finite() {
        aph_443_val / aphstar_443_val
    } else {
        flags |= 0x20; // Set chlorophyll calculation error flag
        0.0
    };

    // Convert maps to vectors for result
    let rrs_vec: Vec<f64> = wavelengths
        .iter()
        .map(|&wl| *rrs.get(&wl).unwrap())
        .collect();
    let u_vec: Vec<f64> = wavelengths.iter().map(|&wl| *u.get(&wl).unwrap()).collect();
    let a_vec: Vec<f64> = wavelengths.iter().map(|&wl| *a.get(&wl).unwrap()).collect();
    let aph_vec: Vec<f64> = wavelengths
        .iter()
        .map(|&wl| *aph.get(&wl).unwrap())
        .collect();
    let acdom_vec: Vec<f64> = wavelengths
        .iter()
        .map(|&wl| *acdom.get(&wl).unwrap())
        .collect();
    let bb_vec: Vec<f64> = wavelengths
        .iter()
        .map(|&wl| *bb.get(&wl).unwrap())
        .collect();
    let bbp_vec: Vec<f64> = wavelengths
        .iter()
        .map(|&wl| bb.get(&wl).unwrap() - bbw.get(&wl).unwrap())
        .collect();

    QaaResult {
        wavelengths,
        rrs: rrs_vec,
        u: u_vec,
        a: a_vec,
        aph: aph_vec,
        acdom: acdom_vec,
        bb: bb_vec,
        bbp: bbp_vec,
        flags,
        chla,
        version: "QAA v6".to_string(),
        reference_wl: wvlref,
        spectral_slope_y: y,
        spectral_slope_s: sr,
        aph_ratio_443: x1,
    }
}

// From https://www.ioccg.org/groups/Software_OCA/QAA_v5.pdf
// The 555 nm used in Eqs. 7-10 can be changed to 550 nm (for MODIS) or 560 nm (for MERIS) without
// causing significant impacts on final IOP results.
