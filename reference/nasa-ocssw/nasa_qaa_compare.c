#include <math.h>
#include <stdio.h>
#include <stdlib.h>

/* NASA OCSSW QAA v6 implementation for comparison */

typedef struct {
  double wavelengths[5];
  double rrs[5];
  double u[5];
  double a[5];
  double aph[5];
  double adg[5];
  double bb[5];
  double bbp[5];
  int flags;
  double chla;
  int reference_wl_idx;
  double spectral_slope_y;
  double spectral_slope_s;
} qaa_result_t;

/* NASA OCSSW constants */
static const double g0 = 0.089;
static const double g1 = 0.125;
static const double acoefs[3] = {-1.146, -1.366, -0.469};

/* Water absorption coefficients at standard wavelengths */
static const double aw[5] = {0.00455, 0.00635, 0.0150, 0.0596, 0.439};

/* Water backscattering coefficients */
static const double bbw[5] = {0.00144, 0.00105, 0.000619, 0.000275, 8.28e-05};

/* Specific phytoplankton absorption */
static const double aphstar[5] = {0.063, 0.0632, 0.0495, 0.0267, 0.00532};

/* Standard wavelengths for SeaWiFS/MODIS */
static const double lambda[5] = {410, 443, 490, 555, 670};

/* QAA v6 main function based on NASA OCSSW */
qaa_result_t qaa_v6_nasa(double *rrs_input) {
  qaa_result_t result = {0};
  int i;

  /* Copy wavelengths */
  for (i = 0; i < 5; i++) {
    result.wavelengths[i] = lambda[i];
  }

  /* Step 0: Convert Rrs to below-water reflectance */
  for (i = 0; i < 5; i++) {
    result.rrs[i] = rrs_input[i] / (0.52 + 1.7 * rrs_input[i]);
  }

  /* Step 1: Calculate u parameter */
  for (i = 0; i < 5; i++) {
    double temp = g0 * g0 + 4.0 * g1 * result.rrs[i];
    result.u[i] = (sqrt(temp) - g0) / (2.0 * g1);
  }

  /* Step 2: Calculate reference absorption at 555nm (index 3) */
  int ref_idx = 3; /* 555nm */
  result.reference_wl_idx = ref_idx;

  double numer = result.rrs[1] + result.rrs[2]; /* 443 + 490 */
  double denom = result.rrs[3] + 5.0 * result.rrs[4] * result.rrs[4] /
                                     result.rrs[2]; /* 555 + 5*(670^2)/490 */

  double aux = log10(numer / denom);
  double rho = acoefs[0] + acoefs[1] * aux + acoefs[2] * aux * aux;
  double aref = aw[ref_idx] + pow(10.0, rho);

  /* Step 3: Calculate reference backscattering */
  double bbpref =
      result.u[ref_idx] * aref / (1.0 - result.u[ref_idx]) - bbw[ref_idx];

  if (bbpref < 0.0) {
    result.flags |= 0x02;
    bbpref = 0.001;
  }

  /* Step 4: Calculate spectral slope Y */
  double rat = result.rrs[1] / result.rrs[3]; /* 443/555 */
  double y = 2.0 * (1.0 - 1.2 * exp(-0.9 * rat));
  if (y < 0.0)
    y = 0.0;
  if (y > 3.0)
    y = 3.0;
  result.spectral_slope_y = y;

  /* Step 5: Calculate total backscattering */
  for (i = 0; i < 5; i++) {
    result.bbp[i] = bbpref * pow(lambda[ref_idx] / lambda[i], y);
    result.bb[i] = result.bbp[i] + bbw[i];
  }

  /* Step 6: Calculate total absorption */
  for (i = 0; i < 5; i++) {
    result.a[i] = (1.0 - result.u[i]) * result.bb[i] / result.u[i];
  }

  /* Step 7: Calculate symbol coefficient */
  double symbol = 0.74 + 0.2 / (0.8 + rat);

  /* Step 8: Calculate spectral slope Sr */
  double sr = 0.015 + 0.002 / (0.6 + rat);
  result.spectral_slope_s = sr;
  double zeta = exp(sr * (443.0 - 410.0));

  /* Step 9: Decompose absorption */
  double denom_decomp = zeta - symbol;
  if (fabs(denom_decomp) < 1e-10) {
    result.flags |= 0x04;
    denom_decomp = 1e-10;
  }

  double dif1 = result.a[0] - symbol * result.a[1]; /* a410 - symbol * a443 */
  double dif2 = aw[0] - symbol * aw[1];
  double adg443 = (dif1 - dif2) / denom_decomp;

  /* Calculate adg at all wavelengths */
  for (i = 0; i < 5; i++) {
    result.adg[i] = adg443 * exp(sr * (443.0 - lambda[i]));
  }

  /* Calculate aph */
  for (i = 0; i < 5; i++) {
    result.aph[i] = result.a[i] - result.adg[i] - aw[i];
    if (result.aph[i] < 0.0) {
      result.flags |= 0x10;
      result.aph[i] = 0.001;
      result.adg[i] = result.a[i] - 0.001 - aw[i];
      if (result.adg[i] < 0.0)
        result.adg[i] = 0.0;
    }
  }

  /* Check aph proportion at 443nm */
  double x1 = result.aph[1] / result.a[1];
  if (x1 < 0.15 || x1 > 0.6 || !isfinite(x1)) {
    result.flags |= 0x08;
    x1 = -0.8 + 1.4 * (result.a[1] - aw[1]) / (result.a[0] - aw[0]);
    if (x1 < 0.15)
      x1 = 0.15;
    if (x1 > 0.6)
      x1 = 0.6;

    /* Recalculate with corrected proportion */
    double corrected_adg443 = result.a[1] - (result.a[1] * x1) - aw[1];
    for (i = 0; i < 5; i++) {
      result.adg[i] = corrected_adg443 * exp(sr * (443.0 - lambda[i]));
      result.aph[i] = result.a[i] - result.adg[i] - aw[i];
      if (result.aph[i] < 0.0) {
        result.aph[i] = 0.001;
        result.adg[i] = result.a[i] - 0.001 - aw[i];
        if (result.adg[i] < 0.0)
          result.adg[i] = 0.0;
      }
    }
  }

  /* Calculate chlorophyll */
  if (aphstar[1] > 0.0 && isfinite(result.aph[1])) {
    result.chla = result.aph[1] / aphstar[1];
  } else {
    result.flags |= 0x20;
    result.chla = 0.0;
  }

  return result;
}

void print_qaa_result(qaa_result_t *result) {
  printf("NASA QAA v6 Results:\n");
  printf("Wavelengths: ");
  for (int i = 0; i < 5; i++) {
    printf("%.0f ", result->wavelengths[i]);
  }
  printf("\n");

  printf("rrs: ");
  for (int i = 0; i < 5; i++) {
    printf("%.10f ", result->rrs[i]);
  }
  printf("\n");

  printf("u: ");
  for (int i = 0; i < 5; i++) {
    printf("%.10f ", result->u[i]);
  }
  printf("\n");

  printf("a: ");
  for (int i = 0; i < 5; i++) {
    printf("%.10f ", result->a[i]);
  }
  printf("\n");

  printf("aph: ");
  for (int i = 0; i < 5; i++) {
    printf("%.10f ", result->aph[i]);
  }
  printf("\n");

  printf("adg: ");
  for (int i = 0; i < 5; i++) {
    printf("%.10f ", result->adg[i]);
  }
  printf("\n");

  printf("bb: ");
  for (int i = 0; i < 5; i++) {
    printf("%.10f ", result->bb[i]);
  }
  printf("\n");

  printf("bbp: ");
  for (int i = 0; i < 5; i++) {
    printf("%.10f ", result->bbp[i]);
  }
  printf("\n");

  printf("flags: %d\n", result->flags);
  printf("chla: %.10f\n", result->chla);
  printf("reference_wl: %.0f\n", result->wavelengths[result->reference_wl_idx]);
  printf("spectral_slope_y: %.10f\n", result->spectral_slope_y);
  printf("spectral_slope_s: %.10f\n", result->spectral_slope_s);
}

int main() {
  /* Test data from your Rust program */
  double test_rrs[5] = {
      0.001974, /* 410/412 nm */
      0.002570, /* 443 nm */
      0.002974, /* 490/488 nm */
      0.001670, /* 555/547 nm */
      0.000324  /* 670/667 nm */
  };

  printf("Input Rrs values:\n");
  for (int i = 0; i < 5; i++) {
    printf("%.0fnm: %.6f\n", lambda[i], test_rrs[i]);
  }
  printf("\n");

  qaa_result_t result = qaa_v6_nasa(test_rrs);
  print_qaa_result(&result);

  return 0;
}
