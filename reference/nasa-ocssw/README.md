# NASA OCSSW QAA Reference Implementation

This directory contains the reference C implementation of the Quasi-Analytical Algorithm (QAA) v6, based directly on the NASA Ocean Color Science Software (OCSSW) source code.

## Purpose

This reference implementation serves as:
- **Validation baseline** for the Rust QAA implementation in `src/iop/qaa.rs`
- **Comparison tool** to ensure algorithm correctness
- **Documentation** of the exact NASA OCSSW approach

## Files

- `nasa_qaa_compare.c` - C implementation following NASA OCSSW QAA algorithm
- `nasa_qaa_compare` - Compiled executable (if present)
- `README.md` - This documentation

## NASA OCSSW Reference

The implementation is based on the official NASA OCSSW QAA source code:
- **URL**: https://oceancolor.gsfc.nasa.gov/docs/ocssw/qaa_8c_source.html
- **Algorithm**: QAA version 6
- **Wavelengths**: Standard [410, 443, 490, 555, 670] nm
- **Constants**: Exact NASA coefficients and parameters

## Building

```bash
cd reference/nasa-ocssw
gcc -o nasa_qaa_compare nasa_qaa_compare.c -lm
```

## Running

```bash
./nasa_qaa_compare
```

The program runs with hardcoded test data that matches the input used in the Rust implementation.

## Test Data

Uses the same remote sensing reflectance (Rrs) values as the main Rust implementation:
- 410nm: 0.001974
- 443nm: 0.002570
- 490nm: 0.002974
- 555nm: 0.001670
- 670nm: 0.000324

## Expected Output

The program outputs all QAA algorithm results including:
- Wavelengths and converted reflectance values
- U-parameter and absorption coefficients
- Phytoplankton and CDOM absorption
- Backscattering coefficients
- Quality flags and chlorophyll-a concentration
- Spectral slopes and algorithm metadata

## Comparison with Rust Implementation

This C reference can be used to validate the Rust implementation in `src/iop/qaa.rs`. Key comparison points:
- **Flags**: Should match exactly (indicates same quality issues detected)
- **Spectral slopes**: Should be very close (indicates correct algorithm steps)
- **Chlorophyll**: May differ slightly due to different optical coefficient tables
- **Physical bounds**: Both should respect the same NASA QAA constraints

## Algorithm Differences

Note that the Rust implementation includes additional features:
- **Satellite band mapping**: Automatically maps to closest available wavelengths
- **Robust wavelength handling**: Won't crash on missing exact wavelengths
- **Enhanced error handling**: More comprehensive quality flagging

The C reference uses exact NASA wavelengths and will expect precise input format.