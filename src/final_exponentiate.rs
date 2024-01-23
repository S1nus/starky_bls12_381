use std::{str::FromStr, cmp::min};

use itertools::Itertools;
use num_bigint::{BigUint, ToBigUint};
use plonky2::{
    field::{
        extension::{Extendable, FieldExtension},
        packed::PackedField,
        polynomial::PolynomialValues,
        types::Field,
    },
    hash::hash_types::RichField,
    iop::ext_target::ExtensionTarget,
    util::transpose,
};
use starky::{
    constraint_consumer::ConstraintConsumer,
    evaluation_frame::{StarkEvaluationFrame, StarkFrame},
    stark::Stark,
};

use crate::native::{
    add_u32_slices, add_u32_slices_12, get_bits_as_array, get_div_rem_modulus_from_biguint_12,
    get_selector_bits_from_u32, get_u32_vec_from_literal, get_u32_vec_from_literal_24, modulus,
    multiply_by_slice, sub_u32_slices, Fp, Fp2, calc_qs, calc_precomp_stuff_loop0, sub_u32_slices_12,
    mul_u32_slice_u32, mod_inverse, get_bls_12_381_parameter, calc_precomp_stuff_loop1, Fp6, Fp12,
    mul_by_nonresidue, fp4_square,
};

use crate::fp::*;
use crate::fp2::*;
use crate::fp6::*;
use crate::fp12::*;
use crate::utils::*;

pub const FINAL_EXP_ROW_SELECTORS: usize = 0;
pub const FINAL_EXP_FORBENIUS_MAP_SELECTOR: usize = FINAL_EXP_ROW_SELECTORS + 8192;
pub const FINAL_EXP_CYCLOTOMIC_EXP_SELECTOR: usize = FINAL_EXP_FORBENIUS_MAP_SELECTOR + 1;
pub const FINAL_EXP_MUL_SELECTOR: usize = FINAL_EXP_CYCLOTOMIC_EXP_SELECTOR + 1;
pub const FINAL_EXP_CYCLOTOMIC_SQ_SELECTOR: usize = FINAL_EXP_MUL_SELECTOR + 1;
pub const FINAL_EXP_CONJUGATE_SELECTOR: usize = FINAL_EXP_CYCLOTOMIC_SQ_SELECTOR + 1;
pub const FINAL_EXP_INPUT_OFFSET: usize = FINAL_EXP_CONJUGATE_SELECTOR + 1;
pub const FINAL_EXP_T0_OFFSET: usize = FINAL_EXP_INPUT_OFFSET + 12*12;
pub const FINAL_EXP_T1_OFFSET: usize = FINAL_EXP_T0_OFFSET + 12*12;
pub const FINAL_EXP_T2_OFFSET: usize = FINAL_EXP_T1_OFFSET + 12*12;
pub const FINAL_EXP_T3_OFFSET: usize = FINAL_EXP_T2_OFFSET + 12*12;
pub const FINAL_EXP_T4_OFFSET: usize = FINAL_EXP_T3_OFFSET + 12*12;
pub const FINAL_EXP_T5_OFFSET: usize = FINAL_EXP_T4_OFFSET + 12*12;
pub const FINAL_EXP_T6_OFFSET: usize = FINAL_EXP_T5_OFFSET + 12*12;
pub const FINAL_EXP_T7_OFFSET: usize = FINAL_EXP_T6_OFFSET + 12*12;
pub const FINAL_EXP_T8_OFFSET: usize = FINAL_EXP_T7_OFFSET + 12*12;
pub const FINAL_EXP_T9_OFFSET: usize = FINAL_EXP_T8_OFFSET + 12*12;
pub const FINAL_EXP_T10_OFFSET: usize = FINAL_EXP_T9_OFFSET + 12*12;
pub const FINAL_EXP_T11_OFFSET: usize = FINAL_EXP_T10_OFFSET + 12*12;
pub const FINAL_EXP_T12_OFFSET: usize = FINAL_EXP_T11_OFFSET + 12*12;
pub const FINAL_EXP_T13_OFFSET: usize = FINAL_EXP_T12_OFFSET + 12*12;
pub const FINAL_EXP_T14_OFFSET: usize = FINAL_EXP_T13_OFFSET + 12*12;
pub const FINAL_EXP_T15_OFFSET: usize = FINAL_EXP_T14_OFFSET + 12*12;
pub const FINAL_EXP_T16_OFFSET: usize = FINAL_EXP_T15_OFFSET + 12*12;
pub const FINAL_EXP_T17_OFFSET: usize = FINAL_EXP_T16_OFFSET + 12*12;
pub const FINAL_EXP_T18_OFFSET: usize = FINAL_EXP_T17_OFFSET + 12*12;
pub const FINAL_EXP_T19_OFFSET: usize = FINAL_EXP_T18_OFFSET + 12*12;
pub const FINAL_EXP_T20_OFFSET: usize = FINAL_EXP_T19_OFFSET + 12*12;
pub const FINAL_EXP_T21_OFFSET: usize = FINAL_EXP_T20_OFFSET + 12*12;
pub const FINAL_EXP_T22_OFFSET: usize = FINAL_EXP_T21_OFFSET + 12*12;
pub const FINAL_EXP_T23_OFFSET: usize = FINAL_EXP_T22_OFFSET + 12*12;
pub const FINAL_EXP_T24_OFFSET: usize = FINAL_EXP_T23_OFFSET + 12*12;
pub const FINAL_EXP_T25_OFFSET: usize = FINAL_EXP_T24_OFFSET + 12*12;
pub const FINAL_EXP_T26_OFFSET: usize = FINAL_EXP_T25_OFFSET + 12*12;
pub const FINAL_EXP_T27_OFFSET: usize = FINAL_EXP_T26_OFFSET + 12*12;
pub const FINAL_EXP_T28_OFFSET: usize = FINAL_EXP_T27_OFFSET + 12*12;
pub const FINAL_EXP_T29_OFFSET: usize = FINAL_EXP_T28_OFFSET + 12*12;
pub const FINAL_EXP_T30_OFFSET: usize = FINAL_EXP_T29_OFFSET + 12*12;
pub const FINAL_EXP_T31_OFFSET: usize = FINAL_EXP_T30_OFFSET + 12*12;
pub const FINAL_EXP_OP_OFFSET: usize = FINAL_EXP_T31_OFFSET + 12*12;
pub const FINAL_EXP_TOTAL_COLUMNS: usize = FINAL_EXP_OP_OFFSET + CYCLOTOMIC_EXP_TOTAL_COLUMNS;

pub const FP12_MUL_ROWS: usize = 12;
pub const FP12_FORBENIUS_MAP_ROWS: usize = 12;
pub const CYCLOTOMIC_SQ_ROWS: usize = 12;
pub const CONJUGATE_ROWS: usize = 1;
pub const CYCLOTOMIC_EXP_ROWS: usize = 70*12 + 1;

pub const T0_ROW: usize = 0;
pub const T1_ROW: usize = T0_ROW + FP12_FORBENIUS_MAP_ROWS;
pub const T2_ROW: usize = T1_ROW + FP12_MUL_ROWS;
pub const T3_ROW: usize = T2_ROW + FP12_FORBENIUS_MAP_ROWS;
pub const T4_ROW: usize = T3_ROW + FP12_MUL_ROWS;
pub const T5_ROW: usize = T4_ROW + CYCLOTOMIC_EXP_ROWS;
pub const T6_ROW: usize = T5_ROW + CONJUGATE_ROWS;
pub const T7_ROW: usize = T6_ROW + CYCLOTOMIC_SQ_ROWS;
pub const T8_ROW: usize = T7_ROW + CONJUGATE_ROWS;
pub const T9_ROW: usize = T8_ROW + FP12_MUL_ROWS;
pub const T10_ROW: usize = T9_ROW + CYCLOTOMIC_EXP_ROWS;
pub const T11_ROW: usize = T10_ROW + CONJUGATE_ROWS;
pub const T12_ROW: usize = T11_ROW + CYCLOTOMIC_EXP_ROWS;
pub const T13_ROW: usize = T12_ROW + CONJUGATE_ROWS;
pub const T14_ROW: usize = T13_ROW + CYCLOTOMIC_EXP_ROWS;
pub const T15_ROW: usize = T14_ROW + CONJUGATE_ROWS;
pub const T16_ROW: usize = T15_ROW + CYCLOTOMIC_SQ_ROWS;
pub const T17_ROW: usize = T16_ROW + FP12_MUL_ROWS;
pub const T18_ROW: usize = T17_ROW + CYCLOTOMIC_EXP_ROWS;
pub const T19_ROW: usize = T18_ROW + CONJUGATE_ROWS;
pub const T20_ROW: usize = T19_ROW + FP12_MUL_ROWS;
pub const T21_ROW: usize = T20_ROW + FP12_FORBENIUS_MAP_ROWS;
pub const T22_ROW: usize = T21_ROW + FP12_MUL_ROWS;
pub const T23_ROW: usize = T22_ROW + FP12_FORBENIUS_MAP_ROWS;
pub const T24_ROW: usize = T23_ROW + CONJUGATE_ROWS;
pub const T25_ROW: usize = T24_ROW + FP12_MUL_ROWS;
pub const T26_ROW: usize = T25_ROW + FP12_FORBENIUS_MAP_ROWS;
pub const T27_ROW: usize = T26_ROW + CONJUGATE_ROWS;
pub const T28_ROW: usize = T27_ROW + FP12_MUL_ROWS;
pub const T29_ROW: usize = T28_ROW + FP12_MUL_ROWS;
pub const T30_ROW: usize = T29_ROW + FP12_MUL_ROWS;
pub const T31_ROW: usize = T30_ROW + FP12_MUL_ROWS;
pub const TOTAL_ROW: usize = T31_ROW + FP12_MUL_ROWS;

pub const TOTAL_COLUMNS: usize = FINAL_EXP_TOTAL_COLUMNS;
pub const COLUMNS: usize = TOTAL_COLUMNS;

pub const PIS_INPUT_OFFSET: usize = 0;
pub const PIS_OUTPUT_OFFSET: usize = PIS_INPUT_OFFSET + 24*3*2;
pub const PUBLIC_INPUTS: usize = PIS_OUTPUT_OFFSET + 24*3*2;

// A (Fp) * B (Fp) => C (Fp)
#[derive(Clone, Copy)]
pub struct FinalExponentiateStark<F: RichField + Extendable<D>, const D: usize> {
    num_rows: usize,
    _f: std::marker::PhantomData<F>,
}

pub fn fill_trace_forbenius<F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(trace: &mut Vec<[F; C]>, x: &Fp12, pow: usize, start_row: usize, end_row: usize, output_col: usize) -> Fp12 {
    let res = x.forbenius_map(pow);
    for row in start_row..end_row+1 {
        trace[row][FINAL_EXP_FORBENIUS_MAP_SELECTOR] = F::ONE;
    }
    for row in 0..trace.len() {
        assign_u32_in_series(trace, row, output_col, &res.get_u32_slice().concat());
    }
    fill_trace_fp12_forbenius_map(trace, x, pow, start_row, end_row, FINAL_EXP_OP_OFFSET);
    res
}

pub fn fill_trace_mul<F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(trace: &mut Vec<[F; C]>, x: &Fp12, y: &Fp12, start_row: usize, end_row: usize, output_col: usize) -> Fp12 {
    let res = (*x)*(*y);
    for row in start_row..end_row+1 {
        trace[row][FINAL_EXP_MUL_SELECTOR] = F::ONE;
    }
    for row in 0..trace.len() {
        assign_u32_in_series(trace, row, output_col, &res.get_u32_slice().concat());
    }
    fill_trace_fp12_multiplication(trace, &x, &y, start_row, end_row, FINAL_EXP_OP_OFFSET);
    res
}

pub fn fill_trace_div<F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(trace: &mut Vec<[F; C]>, x: &Fp12, y: &Fp12, start_row: usize, end_row: usize, output_col: usize) -> Fp12 {
    let res = *x / *y;
    for row in start_row..end_row+1 {
        trace[row][FINAL_EXP_MUL_SELECTOR] = F::ONE;
    }
    for row in 0..trace.len() {
        assign_u32_in_series(trace, row, output_col, &res.get_u32_slice().concat());
    }
    fill_trace_fp12_multiplication(trace, &res, &y, start_row, end_row, FINAL_EXP_OP_OFFSET);
    res
}

pub fn fill_trace_cyc_exp<F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(trace: &mut Vec<[F; C]>, x: &Fp12, start_row: usize, end_row: usize, output_col: usize) -> Fp12 {
    let res = x.cyclotocmicExponent();
    for row in start_row..end_row+1 {
        trace[row][FINAL_EXP_CYCLOTOMIC_EXP_SELECTOR] = F::ONE;
    }
    for row in 0..trace.len() {
        assign_u32_in_series(trace, row, output_col, &res.get_u32_slice().concat());
    }
    fill_trace_cyclotomic_exp(trace, x, start_row, end_row, FINAL_EXP_OP_OFFSET);
    res
}

pub fn fill_trace_conjugate<F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(trace: &mut Vec<[F; C]>, x: &Fp12, row: usize, output_col: usize) -> Fp12 {
    let res = x.conjugate();
    trace[row][FINAL_EXP_CONJUGATE_SELECTOR] = F::ONE;
    for i in 0..trace.len() {
        assign_u32_in_series(trace, i, output_col, &res.get_u32_slice().concat());
    }
    fill_trace_fp12_conjugate(trace, x, row, FINAL_EXP_OP_OFFSET);
    res
}

pub fn fill_trace_cyc_sq<F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(trace: &mut Vec<[F; C]>, x: &Fp12, start_row: usize, end_row: usize, output_col: usize) -> Fp12 {
    let res = x.cyclotomicSquare();
    for row in start_row..end_row+1 {
        trace[row][FINAL_EXP_CYCLOTOMIC_SQ_SELECTOR] = F::ONE;
    }
    for row in 0..trace.len() {
        assign_u32_in_series(trace, row, output_col, &res.get_u32_slice().concat());
    }
    fill_trace_cyclotomic_sq(trace, x, start_row, end_row, FINAL_EXP_OP_OFFSET);
    res
}

// Implement trace generator
impl<F: RichField + Extendable<D>, const D: usize> FinalExponentiateStark<F, D> {
    pub fn new(num_rows: usize) -> Self {
        Self {
            num_rows,
            _f: std::marker::PhantomData,
        }
    }

    pub fn generate_trace(&self, x: Fp12) -> Vec<[F; TOTAL_COLUMNS]> {
        let mut trace = vec![[F::ZERO; TOTAL_COLUMNS]; self.num_rows];
        for row in 0..trace.len() {
            trace[row][FINAL_EXP_ROW_SELECTORS + row] = F::ONE;
            assign_u32_in_series(&mut trace, row, FINAL_EXP_INPUT_OFFSET, &x.get_u32_slice().concat());
        }
        let t0 = fill_trace_forbenius(&mut trace, &x, 6, T0_ROW, T1_ROW-1, FINAL_EXP_T0_OFFSET);
        let t1 = fill_trace_div(&mut trace, &t0, &x, T1_ROW, T2_ROW-1, FINAL_EXP_T1_OFFSET);
        let t2 = fill_trace_forbenius(&mut trace, &t1, 2, T2_ROW, T3_ROW-1, FINAL_EXP_T2_OFFSET);
        let t3 = fill_trace_mul(&mut trace, &t2, &t1, T3_ROW, T4_ROW-1, FINAL_EXP_T3_OFFSET);
        let t4 = fill_trace_cyc_exp(&mut trace, &t3, T4_ROW, T5_ROW-1, FINAL_EXP_T4_OFFSET);
        let t5 = fill_trace_conjugate(&mut trace, &t4, T5_ROW, FINAL_EXP_T5_OFFSET);
        let t6 = fill_trace_cyc_sq(&mut trace, &t3, T6_ROW, T7_ROW-1, FINAL_EXP_T6_OFFSET);
        let t7 = fill_trace_conjugate(&mut trace, &t6, T7_ROW, FINAL_EXP_T7_OFFSET);
        let t8 = fill_trace_mul(&mut trace, &t7, &t5, T8_ROW, T9_ROW-1, FINAL_EXP_T8_OFFSET);
        let t9 = fill_trace_cyc_exp(&mut trace, &t8, T9_ROW, T10_ROW-1, FINAL_EXP_T9_OFFSET);
        let t10 = fill_trace_conjugate(&mut trace, &t9, T10_ROW, FINAL_EXP_T10_OFFSET);
        let t11 = fill_trace_cyc_exp(&mut trace, &t10, T11_ROW, T12_ROW-1, FINAL_EXP_T11_OFFSET);
        let t12 = fill_trace_conjugate(&mut trace, &t11, T12_ROW, FINAL_EXP_T12_OFFSET);
        let t13 = fill_trace_cyc_exp(&mut trace, &t12, T13_ROW, T14_ROW-1, FINAL_EXP_T13_OFFSET);
        let t14 = fill_trace_conjugate(&mut trace, &t13, T14_ROW, FINAL_EXP_T14_OFFSET);
        let t15 = fill_trace_cyc_sq(&mut trace, &t5, T15_ROW, T16_ROW-1, FINAL_EXP_T15_OFFSET);
        let t16 = fill_trace_mul(&mut trace, &t14, &t15, T16_ROW, T17_ROW-1, FINAL_EXP_T16_OFFSET);
        let t17 = fill_trace_cyc_exp(&mut trace, &t16, T17_ROW, T18_ROW-1, FINAL_EXP_T17_OFFSET);
        let t18 = fill_trace_conjugate(&mut trace, &t17, T18_ROW, FINAL_EXP_T18_OFFSET);
        let t19 = fill_trace_mul(&mut trace, &t5, &t12, T19_ROW, T20_ROW-1, FINAL_EXP_T19_OFFSET);
        let t20 = fill_trace_forbenius(&mut trace, &t19, 2, T20_ROW, T21_ROW-1, FINAL_EXP_T20_OFFSET);
        let t21 = fill_trace_mul(&mut trace, &t10, &t3, T21_ROW, T22_ROW-1, FINAL_EXP_T21_OFFSET);
        let t22 = fill_trace_forbenius(&mut trace, &t21, 3, T22_ROW, T23_ROW-1, FINAL_EXP_T22_OFFSET);
        let t23 = fill_trace_conjugate(&mut trace, &t3, T23_ROW, FINAL_EXP_T23_OFFSET);
        let t24 = fill_trace_mul(&mut trace, &t16, &t23, T24_ROW, T25_ROW-1, FINAL_EXP_T24_OFFSET);
        let t25 = fill_trace_forbenius(&mut trace, &t24, 1, T25_ROW, T26_ROW-1, FINAL_EXP_T25_OFFSET);
        let t26 = fill_trace_conjugate(&mut trace, &t8, T26_ROW, FINAL_EXP_T26_OFFSET);
        let t27 = fill_trace_mul(&mut trace, &t18, &t26, T27_ROW, T28_ROW-1, FINAL_EXP_T27_OFFSET);
        let t28 = fill_trace_mul(&mut trace, &t27, &t3, T28_ROW, T29_ROW-1, FINAL_EXP_T28_OFFSET);
        let t29 = fill_trace_mul(&mut trace, &t20, &t22, T29_ROW, T30_ROW-1, FINAL_EXP_T29_OFFSET);
        let t30 = fill_trace_mul(&mut trace, &t29, &t25, T30_ROW, T31_ROW-1, FINAL_EXP_T30_OFFSET);
        let t31 = fill_trace_mul(&mut trace, &t30, &t28, T31_ROW, TOTAL_ROW-1, FINAL_EXP_T31_OFFSET);
        trace
    }
}

pub fn trace_rows_to_poly_values<F: Field>(
    trace_rows: Vec<[F; TOTAL_COLUMNS]>,
) -> Vec<PolynomialValues<F>> {
    let trace_row_vecs = trace_rows.into_iter().map(|row| row.to_vec()).collect_vec();
    let trace_col_vecs: Vec<Vec<F>> = transpose(&trace_row_vecs);
    trace_col_vecs
        .into_iter()
        .map(|column| PolynomialValues::new(column))
        .collect()
}

fn add_constraints_forbenius<F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize
>(
    local_values: &[P],
    next_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    row: usize,
    input_col: usize,
    output_col: usize,
    pow: usize,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    for i in row..row + FP12_FORBENIUS_MAP_ROWS {
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            (local_values[FINAL_EXP_FORBENIUS_MAP_SELECTOR] - P::ONES)
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_CYCLOTOMIC_EXP_SELECTOR]
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_MUL_SELECTOR]
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_CYCLOTOMIC_SQ_SELECTOR]
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_CONJUGATE_SELECTOR]
        );
    }
    for i in 0..24*3*2 {
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + row] *
            (local_values[input_col + i] -
            local_values[FINAL_EXP_OP_OFFSET + FP12_FORBENIUS_MAP_INPUT_OFFSET + i])
        );
    }
    yield_constr.constraint(
        local_values[FINAL_EXP_ROW_SELECTORS + row] *
        (local_values[FINAL_EXP_OP_OFFSET + FP12_FORBENIUS_MAP_POW_OFFSET] - FE::from_canonical_usize(pow))
    );
    for i in 0..12 {
        for j in 0..12 {
            let offset = if j == 0 {
                FP12_FORBENIUS_MAP_R0_CALC_OFFSET + FP6_FORBENIUS_MAP_X_CALC_OFFSET + FP2_FORBENIUS_MAP_INPUT_OFFSET
            } else if j == 1 {
                FP12_FORBENIUS_MAP_R0_CALC_OFFSET + FP6_FORBENIUS_MAP_X_CALC_OFFSET + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCED_OFFSET
            } else if j == 2 {
                FP12_FORBENIUS_MAP_R0_CALC_OFFSET + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET
            } else if j == 3 {
                FP12_FORBENIUS_MAP_R0_CALC_OFFSET + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET
            } else if j == 4 {
                FP12_FORBENIUS_MAP_R0_CALC_OFFSET + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET
            } else if j == 5 {
                FP12_FORBENIUS_MAP_R0_CALC_OFFSET + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET
            } else if j == 6 {
                FP12_FORBENIUS_MAP_C0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET
            } else if j == 7 {
                FP12_FORBENIUS_MAP_C0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET
            } else if j == 8 {
                FP12_FORBENIUS_MAP_C1_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET
            } else if j == 9 {
                FP12_FORBENIUS_MAP_C1_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET
            } else if j == 10 {
                FP12_FORBENIUS_MAP_C2_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET
            } else {
                FP12_FORBENIUS_MAP_C2_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET
            };
            yield_constr.constraint(
                local_values[FINAL_EXP_ROW_SELECTORS + row] *
                (local_values[FINAL_EXP_OP_OFFSET + offset + i] -
                local_values[output_col + j*12 + i])
            );
        }
    }
}

fn add_constraints_mul<F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize
>(
    local_values: &[P],
    next_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    row: usize,
    x_col: usize,
    y_col: usize,
    res_col: usize,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    for i in row..row + FP12_MUL_ROWS {
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_FORBENIUS_MAP_SELECTOR]
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_CYCLOTOMIC_EXP_SELECTOR]
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            (local_values[FINAL_EXP_MUL_SELECTOR] - P::ONES)
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_CYCLOTOMIC_SQ_SELECTOR]
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_CONJUGATE_SELECTOR]
        );
    }
    for i in 0..24*3*2 {
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + row] *
            (local_values[x_col + i] -
            local_values[FINAL_EXP_OP_OFFSET + FP12_MUL_X_INPUT_OFFSET + i])
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + row] *
            (local_values[y_col + i] -
            local_values[FINAL_EXP_OP_OFFSET + FP12_MUL_Y_INPUT_OFFSET + i])
        );
    }
    for i in 0..12 {
        for j in 0..6 {
            for k in 0..2 {
                let x_y = if k == 0 {
                    FP12_MUL_X_CALC_OFFSET + FP6_ADDITION_TOTAL
                } else {
                    FP12_MUL_Y_CALC_OFFSET + FP6_ADDITION_TOTAL + FP6_SUBTRACTION_TOTAL
                };
                let offset = x_y + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL)*j + FP_SINGLE_REDUCED_OFFSET + i;
                yield_constr.constraint(
                    local_values[FINAL_EXP_ROW_SELECTORS + row] *
                    (local_values[res_col + k*24*3 + j*12 + i] -
                    local_values[FINAL_EXP_OP_OFFSET + offset])
                );
            }
        }
    }
}

fn add_constraints_cyc_exp<F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize
>(
    local_values: &[P],
    next_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    row: usize,
    input_col: usize,
    output_col: usize,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    for i in row..row + CYCLOTOMIC_EXP_ROWS {
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_FORBENIUS_MAP_SELECTOR]
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            (local_values[FINAL_EXP_CYCLOTOMIC_EXP_SELECTOR] - P::ONES)
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_MUL_SELECTOR]
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_CYCLOTOMIC_SQ_SELECTOR]
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_CONJUGATE_SELECTOR]
        );
    }
    for i in 0..24*3*2 {
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + row] *
            (local_values[input_col + i] -
            local_values[FINAL_EXP_OP_OFFSET + INPUT_OFFSET + i])
        );
    }
    for i in 0..24*3*2 {
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + row + CYCLOTOMIC_EXP_ROWS - 1] *
            local_values[FINAL_EXP_OP_OFFSET + RES_ROW_SELECTOR_OFFSET] *
            (local_values[output_col + i] -
            local_values[FINAL_EXP_OP_OFFSET + Z_OFFSET + i])
        );
    }
}

pub fn add_constraints_conjugate<F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize
>(
    local_values: &[P],
    next_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    row: usize,
    input_col: usize,
    output_col: usize,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    yield_constr.constraint(
        local_values[FINAL_EXP_ROW_SELECTORS + row] *
        local_values[FINAL_EXP_FORBENIUS_MAP_SELECTOR]
    );
    yield_constr.constraint(
        local_values[FINAL_EXP_ROW_SELECTORS + row] *
        local_values[FINAL_EXP_CYCLOTOMIC_EXP_SELECTOR]
    );
    yield_constr.constraint(
        local_values[FINAL_EXP_ROW_SELECTORS + row] *
        local_values[FINAL_EXP_MUL_SELECTOR]
    );
    yield_constr.constraint(
        local_values[FINAL_EXP_ROW_SELECTORS + row] *
        local_values[FINAL_EXP_CYCLOTOMIC_SQ_SELECTOR]
    );
    yield_constr.constraint(
        local_values[FINAL_EXP_ROW_SELECTORS + row] *
        (local_values[FINAL_EXP_CONJUGATE_SELECTOR] - P::ONES)
    );
    for i in 0..24*3*2 {
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + row] *
            (local_values[input_col + i] -
            local_values[FINAL_EXP_OP_OFFSET + FP12_CONJUGATE_INPUT_OFFSET + i])
        );
    }
    for i in 0..24*3*2 {
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + row] *
            (local_values[output_col + i] -
            local_values[FINAL_EXP_OP_OFFSET + FP12_CONJUGATE_OUTPUT_OFFSET + i])
        );
    }
}

pub fn add_constraints_cyc_sq<F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize
>(
    local_values: &[P],
    next_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    row: usize,
    input_col: usize,
    output_col: usize,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    for i in row..row + CYCLOTOMIC_SQ_ROWS {
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_FORBENIUS_MAP_SELECTOR]
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_CYCLOTOMIC_EXP_SELECTOR]
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_MUL_SELECTOR]
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            (local_values[FINAL_EXP_CYCLOTOMIC_SQ_SELECTOR] - P::ONES)
        );
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + i] *
            local_values[FINAL_EXP_CONJUGATE_SELECTOR]
        );
    }
    for i in 0..24*3*2 {
        yield_constr.constraint(
            local_values[FINAL_EXP_ROW_SELECTORS + row] *
            (local_values[input_col + i] -
            local_values[FINAL_EXP_OP_OFFSET + CYCLOTOMIC_SQ_INPUT_OFFSET + i])
        );
    }
    for i in 0..12 {
        for j in 0..6 {
            let c_offset = if j == 0 {
                CYCLOTOMIC_SQ_C0_CALC_OFFSET
            } else if j == 1 {
                CYCLOTOMIC_SQ_C1_CALC_OFFSET
            } else if j == 2 {
                CYCLOTOMIC_SQ_C2_CALC_OFFSET
            } else if j == 3 {
                CYCLOTOMIC_SQ_C3_CALC_OFFSET
            } else if j == 4 {
                CYCLOTOMIC_SQ_C4_CALC_OFFSET
            } else {
                CYCLOTOMIC_SQ_C5_CALC_OFFSET
            };
            for k in 0..2 {
                let offset = c_offset + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL)*k + FP_SINGLE_REDUCED_OFFSET;
                yield_constr.constraint(
                    local_values[FINAL_EXP_ROW_SELECTORS + row] *
                    (local_values[FINAL_EXP_OP_OFFSET + offset + i] -
                    local_values[output_col + j*24 + k*12 + i])
                );
            }
        }
    }
}

// Implement constraint generator
impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for FinalExponentiateStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, COLUMNS, PUBLIC_INPUTS>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        vars: &Self::EvaluationFrame<FE, P, D2>,
        yield_constr: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let local_values = vars.get_local_values();
        let next_values = vars.get_next_values();
        let public_inputs = vars.get_public_inputs();

        // ----
        for i in 0..24*3*2 {
            yield_constr.constraint(
                local_values[FINAL_EXP_INPUT_OFFSET + i] -
                public_inputs[PIS_INPUT_OFFSET + i]
            );
            yield_constr.constraint(
                local_values[FINAL_EXP_T31_OFFSET + i] -
                public_inputs[PIS_OUTPUT_OFFSET + i]
            );
        }

        for i in 0..self.num_rows {
            let val = if i == 0 {
                P::ONES
            } else {
                P::ZEROS
            };
            yield_constr.constraint_first_row(
                local_values[FINAL_EXP_ROW_SELECTORS + i] - val
            );
        }
        for i in 0..self.num_rows-1 {
            yield_constr.constraint_transition(
                local_values[FINAL_EXP_ROW_SELECTORS + i] -
                next_values[FINAL_EXP_ROW_SELECTORS + i + 1]
            );
        }
        for i in 0..self.num_rows {
            let val = if i == self.num_rows-1 {
                P::ONES
            } else {
                P::ZEROS
            };
            yield_constr.constraint_last_row(
                local_values[FINAL_EXP_ROW_SELECTORS + i] - val
            );
        }

        for i in 0..24*3*2 {
            yield_constr.constraint_transition(
                local_values[FINAL_EXP_INPUT_OFFSET + i] -
                next_values[FINAL_EXP_INPUT_OFFSET + i]
            );
            for j in 0..32 {
                let t = if j == 0 {
                    FINAL_EXP_T0_OFFSET
                } else if j == 1 {
                    FINAL_EXP_T1_OFFSET
                } else if j == 2 {
                    FINAL_EXP_T2_OFFSET
                } else if j == 3 {
                    FINAL_EXP_T3_OFFSET
                } else if j == 4 {
                    FINAL_EXP_T4_OFFSET
                } else if j == 5 {
                    FINAL_EXP_T5_OFFSET
                } else if j == 6 {
                    FINAL_EXP_T6_OFFSET
                } else if j == 7 {
                    FINAL_EXP_T7_OFFSET
                } else if j == 8 {
                    FINAL_EXP_T8_OFFSET
                } else if j == 9 {
                    FINAL_EXP_T9_OFFSET
                } else if j == 10 {
                    FINAL_EXP_T10_OFFSET
                } else if j == 11 {
                    FINAL_EXP_T11_OFFSET
                } else if j == 12 {
                    FINAL_EXP_T12_OFFSET
                } else if j == 13 {
                    FINAL_EXP_T13_OFFSET
                } else if j == 14 {
                    FINAL_EXP_T14_OFFSET
                } else if j == 15 {
                    FINAL_EXP_T15_OFFSET
                } else if j == 16 {
                    FINAL_EXP_T16_OFFSET
                } else if j == 17 {
                    FINAL_EXP_T17_OFFSET
                } else if j == 18 {
                    FINAL_EXP_T18_OFFSET
                } else if j == 19 {
                    FINAL_EXP_T19_OFFSET
                } else if j == 20 {
                    FINAL_EXP_T20_OFFSET
                } else if j == 21 {
                    FINAL_EXP_T21_OFFSET
                } else if j == 22 {
                    FINAL_EXP_T22_OFFSET
                } else if j == 23 {
                    FINAL_EXP_T23_OFFSET
                } else if j == 24 {
                    FINAL_EXP_T24_OFFSET
                } else if j == 25 {
                    FINAL_EXP_T25_OFFSET
                } else if j == 26 {
                    FINAL_EXP_T26_OFFSET
                } else if j == 27 {
                    FINAL_EXP_T27_OFFSET
                } else if j == 28 {
                    FINAL_EXP_T28_OFFSET
                } else if j == 29 {
                    FINAL_EXP_T29_OFFSET
                } else if j == 30 {
                    FINAL_EXP_T30_OFFSET
                } else {
                    FINAL_EXP_T31_OFFSET
                };
                yield_constr.constraint_transition(
                    local_values[t+i] - next_values[t+i]
                );
            }
        }

        // T0
        add_constraints_forbenius(local_values, next_values, yield_constr, T0_ROW, FINAL_EXP_INPUT_OFFSET, FINAL_EXP_T0_OFFSET, 6);

        // T1
        add_constraints_mul(local_values, next_values, yield_constr, T1_ROW, FINAL_EXP_T1_OFFSET, FINAL_EXP_INPUT_OFFSET, FINAL_EXP_T0_OFFSET);

        // T2
        add_constraints_forbenius(local_values, next_values, yield_constr, T2_ROW, FINAL_EXP_T1_OFFSET, FINAL_EXP_T2_OFFSET, 2);

        // T3
        add_constraints_mul(local_values, next_values, yield_constr, T3_ROW, FINAL_EXP_T2_OFFSET, FINAL_EXP_T1_OFFSET, FINAL_EXP_T3_OFFSET);

        // T4
        add_constraints_cyc_exp(local_values, next_values, yield_constr, T4_ROW, FINAL_EXP_T3_OFFSET, FINAL_EXP_T4_OFFSET);

        // T5
        add_constraints_conjugate(local_values, next_values, yield_constr, T5_ROW, FINAL_EXP_T4_OFFSET, FINAL_EXP_T5_OFFSET);

        // T6
        add_constraints_cyc_sq(local_values, next_values, yield_constr, T6_ROW, FINAL_EXP_T3_OFFSET, FINAL_EXP_T6_OFFSET);

        // T7
        add_constraints_conjugate(local_values, next_values, yield_constr, T7_ROW, FINAL_EXP_T6_OFFSET, FINAL_EXP_T7_OFFSET);

        // T8
        add_constraints_mul(local_values, next_values, yield_constr, T8_ROW, FINAL_EXP_T7_OFFSET, FINAL_EXP_T5_OFFSET, FINAL_EXP_T8_OFFSET);

        // T9
        add_constraints_cyc_exp(local_values, next_values, yield_constr, T9_ROW, FINAL_EXP_T8_OFFSET, FINAL_EXP_T9_OFFSET);

        // T10
        add_constraints_conjugate(local_values, next_values, yield_constr, T10_ROW, FINAL_EXP_T9_OFFSET, FINAL_EXP_T10_OFFSET);

        // T11
        add_constraints_cyc_exp(local_values, next_values, yield_constr, T11_ROW, FINAL_EXP_T10_OFFSET, FINAL_EXP_T11_OFFSET);

        // T12
        add_constraints_conjugate(local_values, next_values, yield_constr, T12_ROW, FINAL_EXP_T11_OFFSET, FINAL_EXP_T12_OFFSET);

        // T13
        add_constraints_cyc_exp(local_values, next_values, yield_constr, T13_ROW, FINAL_EXP_T12_OFFSET, FINAL_EXP_T13_OFFSET);

        // T14
        add_constraints_conjugate(local_values, next_values, yield_constr, T14_ROW, FINAL_EXP_T13_OFFSET, FINAL_EXP_T14_OFFSET);

        // T15
        add_constraints_cyc_sq(local_values, next_values, yield_constr, T15_ROW, FINAL_EXP_T5_OFFSET, FINAL_EXP_T15_OFFSET);

        // T16
        add_constraints_mul(local_values, next_values, yield_constr, T16_ROW, FINAL_EXP_T14_OFFSET, FINAL_EXP_T15_OFFSET, FINAL_EXP_T16_OFFSET);

        // T17
        add_constraints_cyc_exp(local_values, next_values, yield_constr, T17_ROW, FINAL_EXP_T16_OFFSET, FINAL_EXP_T17_OFFSET);

        // T18
        add_constraints_conjugate(local_values, next_values, yield_constr, T18_ROW, FINAL_EXP_T17_OFFSET, FINAL_EXP_T18_OFFSET);

        // T19
        add_constraints_mul(local_values, next_values, yield_constr, T19_ROW, FINAL_EXP_T5_OFFSET, FINAL_EXP_T12_OFFSET, FINAL_EXP_T19_OFFSET);

        // T20
        add_constraints_forbenius(local_values, next_values, yield_constr, T20_ROW, FINAL_EXP_T19_OFFSET, FINAL_EXP_T20_OFFSET, 2);

        // T21
        add_constraints_mul(local_values, next_values, yield_constr, T21_ROW, FINAL_EXP_T10_OFFSET, FINAL_EXP_T3_OFFSET, FINAL_EXP_T21_OFFSET);

        // T22
        add_constraints_forbenius(local_values, next_values, yield_constr, T22_ROW, FINAL_EXP_T21_OFFSET, FINAL_EXP_T22_OFFSET, 3);

        // T23
        add_constraints_conjugate(local_values, next_values, yield_constr, T23_ROW, FINAL_EXP_T3_OFFSET, FINAL_EXP_T23_OFFSET);

        // T24
        add_constraints_mul(local_values, next_values, yield_constr, T24_ROW, FINAL_EXP_T16_OFFSET, FINAL_EXP_T23_OFFSET, FINAL_EXP_T24_OFFSET);

        // T25
        add_constraints_forbenius(local_values, next_values, yield_constr, T25_ROW, FINAL_EXP_T24_OFFSET, FINAL_EXP_T25_OFFSET, 1);

        // T26
        add_constraints_conjugate(local_values, next_values, yield_constr, T26_ROW, FINAL_EXP_T8_OFFSET, FINAL_EXP_T26_OFFSET);

        // T27
        add_constraints_mul(local_values, next_values, yield_constr, T27_ROW, FINAL_EXP_T18_OFFSET, FINAL_EXP_T26_OFFSET, FINAL_EXP_T27_OFFSET);

        // T28
        add_constraints_mul(local_values, next_values, yield_constr, T28_ROW, FINAL_EXP_T27_OFFSET, FINAL_EXP_T3_OFFSET, FINAL_EXP_T28_OFFSET);

        // T29
        add_constraints_mul(local_values, next_values, yield_constr, T29_ROW, FINAL_EXP_T20_OFFSET, FINAL_EXP_T22_OFFSET, FINAL_EXP_T29_OFFSET);

        // T30
        add_constraints_mul(local_values, next_values, yield_constr, T30_ROW, FINAL_EXP_T29_OFFSET, FINAL_EXP_T25_OFFSET, FINAL_EXP_T30_OFFSET);

        // T31
        add_constraints_mul(local_values, next_values, yield_constr, T31_ROW, FINAL_EXP_T30_OFFSET, FINAL_EXP_T28_OFFSET, FINAL_EXP_T31_OFFSET);

        add_fp12_forbenius_map_constraints(local_values, next_values, yield_constr, FINAL_EXP_OP_OFFSET, Some(local_values[FINAL_EXP_FORBENIUS_MAP_SELECTOR]));
        add_fp12_multiplication_constraints(local_values, next_values, yield_constr, FINAL_EXP_OP_OFFSET, Some(local_values[FINAL_EXP_MUL_SELECTOR]));
        add_cyclotomic_exp_constraints(local_values, next_values, yield_constr, FINAL_EXP_OP_OFFSET, Some(local_values[FINAL_EXP_CYCLOTOMIC_EXP_SELECTOR]));
        add_fp12_conjugate_constraints(local_values, yield_constr, FINAL_EXP_OP_OFFSET, Some(local_values[FINAL_EXP_CONJUGATE_SELECTOR]));
        add_cyclotomic_sq_constraints(local_values, next_values, yield_constr, FINAL_EXP_OP_OFFSET, Some(local_values[FINAL_EXP_CYCLOTOMIC_SQ_SELECTOR]));
    }

    type EvaluationFrameTarget =
        StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, COLUMNS, PUBLIC_INPUTS>;

    fn eval_ext_circuit(
        &self,
        _builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<F, D>,
        _vars: &Self::EvaluationFrameTarget,
        _yield_constr: &mut starky::constraint_consumer::RecursiveConstraintConsumer<F, D>,
    ) {
        todo!()
    }

    fn constraint_degree(&self) -> usize {
        5
    }
}
