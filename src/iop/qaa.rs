use crate::iop::constants;
use crate::sat_bands::{SatBands, Satellites};
use std::collections::BTreeMap;

// Quasi-Analytical Algorithm (QAA v5) for SeaWiFS
//
// Based on Lee et al. (2002) and subsequent updates
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

    // SeaWiFS wavelengths used in the original code (nm)
    let original_wavelengths = [412, 443, 490, 555, 670];

    let sat_bands = SatBands::new(satellite);

    // From lambda, extract wavelengths closest to the original wavelengths
    let wavelengths: Vec<u32> = original_wavelengths
        .iter()
        .map(|&target| sat_bands.closest_band(target))
        .collect();

    // Subset aw, bbw, and aphstar to the closest available wavelengths
    let aw = subset_optical_data(&wavelengths, &constants::AW_ALL);
    let bbw = subset_optical_data(&wavelengths, &constants::BBW_ALL);
    let aphstar = subset_optical_data(&wavelengths, &constants::APHSTAR_ALL);

    let mut rrs = subset_optical_data(&wavelengths, rrs);

    // Convert rrs to below sea level
    rrs.iter_mut()
        .for_each(|(_k, v)| *v = *v / (0.512 + (1.7 * *v)));

    // Step 1: Calculate the diffusion probabilities at each wavelengths
    let u: BTreeMap<u32, f64> = rrs
        .iter()
        .map(|(k, v)| {
            let u = ((constants::G0.powi(2) + 4.0 * constants::G1 * v).sqrt() - constants::G0)
                / (2.0 * constants::G1);

            (*k, u)
        })
        .collect();

    // Step 2: Determine reference wavelength and absorption coefficient
    // Get wavelengths directly from satellite bands to avoid index dependency
    let red_wl = sat_bands.closest_band(670); // ~670 nm
    let green_wl = sat_bands.closest_band(555); // ~555 nm (MODIS: 547, MERIS: 560)
    let blue_wl = sat_bands.closest_band(490); // ~490 nm
    let cyan_wl = sat_bands.closest_band(443); // ~443 nm
    let violet_wl = sat_bands.closest_band(412); // ~412 nm

    let rrs_red = rrs.get(&red_wl).unwrap();
    let (wvlref, _aref, bbpref) = if *rrs_red >= constants::RRS_670_THRESHOLD {
        // Use red band as reference
        let wvlref = red_wl;
        let rrs_blue = rrs.get(&blue_wl).unwrap();
        let rrs_cyan = rrs.get(&cyan_wl).unwrap();

        let aref = aw.get(&wvlref).unwrap() + 0.39 * ((rrs_red / (rrs_blue + rrs_cyan)).powf(1.14));

        // Step 3: Calculate reference backscattering
        let u_ref = u.get(&wvlref).unwrap();
        let bbpref = u_ref * aref / (1.0 - u_ref) - bbw.get(&wvlref).unwrap();

        (wvlref, aref, bbpref)
    } else {
        // Use green band (~555nm) as reference with alternative calculation
        let wvlref = green_wl;
        let rrs_443 = rrs.get(&cyan_wl).unwrap();
        let rrs_490 = rrs.get(&blue_wl).unwrap();
        let rrs_555 = rrs.get(&green_wl).unwrap();

        let numer = rrs_443 + rrs_490;
        let denom = rrs_555 + 5.0 * (rrs_red * rrs_red) / rrs_490;
        let aux = (numer / denom).log10();

        let rho = constants::C1 + constants::C2 * aux + constants::C3 * aux.powi(2);
        let aref = aw.get(&wvlref).unwrap() + 10.0_f64.powf(rho);

        // Step 3: Calculate reference backscattering
        let u_ref = u.get(&wvlref).unwrap();
        let bbpref = u_ref * aref / (1.0 - u_ref) - bbw.get(&wvlref).unwrap();

        (wvlref, aref, bbpref)
    };

    // Step 4: Calculate spectral slope Y
    let rrs_443 = rrs.get(&cyan_wl).unwrap();
    let rrs_555 = rrs.get(&green_wl).unwrap();
    let rat = rrs_443 / rrs_555;
    let y = 2.0 * (1.0 - 1.2 * (-0.9 * rat).exp());

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

    // Step 7: Calculate symbol coefficient
    let symbol = 0.74 + 0.2 / (0.8 + rat);

    // Step 8: Calculate spectral slope Sr
    let sr = constants::S + 0.002 / (0.6 + rat);
    let zeta = (sr * (442.5 - 415.5)).exp();

    // Step 9: Calculate ag440 and decompose absorption
    let denom = zeta - symbol;
    let a_412 = a.get(&violet_wl).unwrap();
    let a_443 = a.get(&cyan_wl).unwrap();
    let aw_412 = aw.get(&violet_wl).unwrap();
    let aw_443 = aw.get(&cyan_wl).unwrap();

    let dif1 = a_412 - symbol * a_443;
    let dif2 = aw_412 - symbol * aw_443;
    let acdom440 = (dif1 - dif2) / denom;

    // Calculate initial adg and aph using helper functions
    let initial_adg = calculate_acdom_absorption(&wavelengths, acdom440, sr, cyan_wl);
    let initial_aph = calculate_phytoplankton_absorption(&wavelengths, &a, &initial_adg, &aw);

    // Check and correct aph at 443nm
    let mut x1 = initial_aph.get(&cyan_wl).unwrap() / a_443;

    // aph proportion should be between 0.15 and 0.6
    if !(0.15..=0.6).contains(&x1) && x1.is_finite() {
        x1 = -0.8 + 1.4 * (a_443 - aw_443) / (a_412 - aw_412);
    }

    // Clamp to boundaries
    x1 = x1.clamp(0.15, 0.6);

    // Recalculate acdom440 based on corrected aph at 443nm
    let corrected_acdom440 = a_443 - (a_443 * x1) - aw_443;

    // Final calculations with corrected acdom440
    let mut acdom = calculate_acdom_absorption(&wavelengths, corrected_acdom440, sr, cyan_wl);
    let mut aph = calculate_phytoplankton_absorption(&wavelengths, &a, &acdom, &aw);

    // Handle negative aph values and ensure physical constraints
    // This is a common issue in QAA at red wavelengths with low reflectance
    for (&wl, aph_val) in aph.iter_mut() {
        if *aph_val < 0.0 {
            // At red wavelengths, QAA can produce unrealistic results
            // Apply physical constraints: a must be >= aw + minimum constituents
            let aw_val = *aw.get(&wl).unwrap();
            let a_val = *a.get(&wl).unwrap();

            if a_val < aw_val {
                // Total absorption is less than water absorption - physically impossible
                // This indicates QAA limitation at this wavelength
                // Set conservative estimates
                let min_aph = 0.001;
                let min_acdom = 0.001;

                *aph_val = min_aph;
                if let Some(acdom_val) = acdom.get_mut(&wl) {
                    *acdom_val = min_acdom;
                }

                // Note: This breaks strict mass conservation but maintains physical realism
                // In practice, red wavelength retrievals are often flagged as unreliable
            } else {
                // Normal case: set aph to small positive value and adjust acdom
                let min_aph = 0.001;
                *aph_val = min_aph;

                let corrected_acdom = a_val - min_aph - aw_val;
                if let Some(acdom_val) = acdom.get_mut(&wl) {
                    *acdom_val = corrected_acdom.max(0.0);
                }
            }
        }
    }

    // Calculate chlorophyll concentration
    let aph_wavelengths = [violet_wl, cyan_wl, blue_wl, green_wl, red_wl];
    let aph_subset = subset_optical_data(&aph_wavelengths, &aph);
    let aphstar_subset = subset_optical_data(&aph_wavelengths, &aphstar);

    let mut chl_ratios: Vec<f64> = aph_wavelengths
        .iter()
        .filter_map(|&wl| {
            let aph_val = aph_subset.get(&wl)?;
            let aphstar_val = aphstar_subset.get(&wl)?;
            if aph_val.is_finite() && *aphstar_val > 0.0 {
                Some(aph_val / aphstar_val)
            } else {
                None
            }
        })
        .collect();

    // Calculate median chlorophyll
    chl_ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let chla = if !chl_ratios.is_empty() {
        if chl_ratios.len() % 2 == 0 {
            (chl_ratios[chl_ratios.len() / 2 - 1] + chl_ratios[chl_ratios.len() / 2]) / 2.0
        } else {
            chl_ratios[chl_ratios.len() / 2]
        }
    } else {
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
        flags: 0, // No quality flags set for now
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
