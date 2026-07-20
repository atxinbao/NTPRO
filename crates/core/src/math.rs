// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Mathematical functions and interpolation utilities.
//!
//! This module provides essential mathematical operations for quantitative trading,
//! including linear and quadratic interpolation functions commonly used in financial
//! data processing and analysis.
//!
//! # Epsilon Values
//!
//! Two epsilon thresholds are used in this module:
//!
//! - **`f64::EPSILON * 2.0` (~4.44e-16):** Used for detecting near-zero denominators
//!   in `linear_weight` and `quad_polynomial` to prevent division instability.
//!   This is a machine-precision threshold.
//!
//! - **`1e-8`:** Used in `quadratic_interpolation` for detecting exact sample points.
//!   This is an application-level threshold appropriate for typical financial data.

use thiserror::Error;

/// Errors returned by checked interpolation functions.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum InterpolationError {
    /// An interpolation input was not finite.
    #[error("All inputs must be finite: parameter={parameter}, index={index:?}")]
    NonFiniteInput {
        /// Name of the invalid input.
        parameter: &'static str,
        /// Element index for slice inputs.
        index: Option<usize>,
    },
    /// The interpolation input did not contain enough points.
    #[error("Need at least {minimum} points for quadratic interpolation, found {actual}")]
    InsufficientPoints {
        /// Minimum number of required points.
        minimum: usize,
        /// Number of supplied points.
        actual: usize,
    },
    /// Abscissa and ordinate lengths differed.
    #[error("xs and ys must have the same length: xs={xs_len}, ys={ys_len}")]
    LengthMismatch {
        /// Number of abscissas.
        xs_len: usize,
        /// Number of ordinates.
        ys_len: usize,
    },
    /// Abscissas were not strictly increasing.
    #[error(
        "Abscissas must be strictly increasing: index {left_index}={left}, index {right_index}={right}"
    )]
    UnsortedAbscissas {
        /// Index of the left abscissa.
        left_index: usize,
        /// Index of the right abscissa.
        right_index: usize,
        /// Value of the left abscissa.
        left: f64,
        /// Value of the right abscissa.
        right: f64,
    },
    /// Two abscissas were too close for stable interpolation.
    #[error(
        "Abscissas are too close for stable interpolation: left={left}, right={right}, diff={diff}, min={minimum}"
    )]
    AbscissasTooClose {
        /// Left abscissa.
        left: f64,
        /// Right abscissa.
        right: f64,
        /// Absolute difference between the abscissas.
        diff: f64,
        /// Minimum accepted difference.
        minimum: f64,
    },
    /// A validated interpolation index could not be read.
    #[error("Interpolation index {index} was outside input length {len}")]
    InvalidIndex {
        /// Requested element index.
        index: usize,
        /// Input length.
        len: usize,
    },
}

/// Macro for approximate floating-point equality comparison.
///
/// This macro compares two floating-point values with a specified epsilon tolerance,
/// providing a safe alternative to exact equality checks which can fail due to
/// floating-point precision issues.
///
/// # Usage
///
/// ```rust
/// use nautilus_core::approx_eq;
///
/// let a = 0.1 + 0.2;
/// let b = 0.3;
/// assert!(approx_eq!(f64, a, b, epsilon = 1e-10));
/// ```
#[macro_export]
macro_rules! approx_eq {
    ($type:ty, $left:expr, $right:expr, epsilon = $epsilon:expr) => {{
        let left_val: $type = $left;
        let right_val: $type = $right;
        (left_val - right_val).abs() < $epsilon
    }};
}

/// Calculates the interpolation weight between `x1` and `x2` for a value `x`.
///
/// The returned weight `w` satisfies `y = (1 - w) * y1 + w * y2` when
/// interpolating ordinates that correspond to abscissas `x1` and `x2`.
///
/// # Panics
///
/// - If any input is NaN or infinite.
/// - If `x1` and `x2` are too close (within machine epsilon), which would
///   cause division by zero or numerical instability.
#[inline]
#[must_use]
pub fn linear_weight(x1: f64, x2: f64, x: f64) -> f64 {
    try_linear_weight(x1, x2, x).unwrap_or_else(|error| panic!("{error}"))
}

/// Calculates an interpolation weight without panicking on invalid inputs.
///
/// # Errors
///
/// Returns [`InterpolationError::NonFiniteInput`] for non-finite values and
/// [`InterpolationError::AbscissasTooClose`] for an unstable denominator.
#[inline]
pub fn try_linear_weight(x1: f64, x2: f64, x: f64) -> Result<f64, InterpolationError> {
    const EPSILON: f64 = f64::EPSILON * 2.0; // ~4.44e-16

    for (parameter, value) in [("x1", x1), ("x2", x2), ("x", x)] {
        if !value.is_finite() {
            return Err(InterpolationError::NonFiniteInput {
                parameter,
                index: None,
            });
        }
    }

    let diff = (x2 - x1).abs();
    if diff < EPSILON {
        return Err(InterpolationError::AbscissasTooClose {
            left: x1,
            right: x2,
            diff,
            minimum: EPSILON,
        });
    }

    Ok((x - x1) / (x2 - x1))
}

/// Performs linear interpolation using a weight factor.
///
/// Given ordinates `y1` and `y2` and a weight `x1_diff`, computes the
/// interpolated value using the formula: `y1 + x1_diff * (y2 - y1)`.
#[inline]
#[must_use]
pub fn linear_weighting(y1: f64, y2: f64, x1_diff: f64) -> f64 {
    x1_diff.mul_add(y2 - y1, y1)
}

/// Finds the position for interpolation in a sorted array.
///
/// Returns the index of the largest element in `xs` that is less than `x`,
/// clamped to the valid range `[0, xs.len() - 1]`.
///
/// # Edge Cases
///
/// - For empty arrays, returns 0
/// - For single-element arrays, always returns index 0, regardless of whether `x > xs[0]`
/// - For values below the minimum, returns 0
/// - For values at or above the maximum, returns `xs.len() - 1`
#[inline]
#[must_use]
pub fn pos_search(x: f64, xs: &[f64]) -> usize {
    if xs.is_empty() {
        return 0;
    }

    let n_elem = xs.len();
    let pos = xs.partition_point(|&val| val < x);
    std::cmp::min(std::cmp::max(pos.saturating_sub(1), 0), n_elem - 1)
}

/// Evaluates the quadratic Lagrange polynomial defined by three points.
///
/// Given points `(x0, y0)`, `(x1, y1)`, `(x2, y2)` this returns *P(x)* where
/// *P* is the unique polynomial of degree ≤ 2 passing through the three
/// points.
///
/// # Panics
///
/// - If any input is NaN or infinite.
/// - If any two abscissas are too close (within machine epsilon), which would
///   cause division by zero or numerical instability.
#[inline]
#[must_use]
pub fn quad_polynomial(x: f64, x0: f64, x1: f64, x2: f64, y0: f64, y1: f64, y2: f64) -> f64 {
    try_quad_polynomial(x, x0, x1, x2, y0, y1, y2).unwrap_or_else(|error| panic!("{error}"))
}

/// Evaluates a quadratic Lagrange polynomial without panicking on invalid inputs.
///
/// # Errors
///
/// Returns [`InterpolationError::NonFiniteInput`] for non-finite values and
/// [`InterpolationError::AbscissasTooClose`] when the polynomial would have an
/// unstable denominator.
#[inline]
#[allow(
    clippy::too_many_arguments,
    reason = "mirrors the existing polynomial API"
)]
pub fn try_quad_polynomial(
    x: f64,
    x0: f64,
    x1: f64,
    x2: f64,
    y0: f64,
    y1: f64,
    y2: f64,
) -> Result<f64, InterpolationError> {
    const EPSILON: f64 = f64::EPSILON * 2.0; // ~4.44e-16

    for (parameter, value) in [
        ("x", x),
        ("x0", x0),
        ("x1", x1),
        ("x2", x2),
        ("y0", y0),
        ("y1", y1),
        ("y2", y2),
    ] {
        if !value.is_finite() {
            return Err(InterpolationError::NonFiniteInput {
                parameter,
                index: None,
            });
        }
    }

    // Protect against coincident x values that would lead to division by zero
    let diff_01 = (x0 - x1).abs();
    let diff_02 = (x0 - x2).abs();
    let diff_12 = (x1 - x2).abs();

    if diff_01 < EPSILON {
        return Err(InterpolationError::AbscissasTooClose {
            left: x0,
            right: x1,
            diff: diff_01,
            minimum: EPSILON,
        });
    }
    if diff_02 < EPSILON {
        return Err(InterpolationError::AbscissasTooClose {
            left: x0,
            right: x2,
            diff: diff_02,
            minimum: EPSILON,
        });
    }
    if diff_12 < EPSILON {
        return Err(InterpolationError::AbscissasTooClose {
            left: x1,
            right: x2,
            diff: diff_12,
            minimum: EPSILON,
        });
    }

    Ok(y0 * (x - x1) * (x - x2) / ((x0 - x1) * (x0 - x2))
        + y1 * (x - x0) * (x - x2) / ((x1 - x0) * (x1 - x2))
        + y2 * (x - x0) * (x - x1) / ((x2 - x0) * (x2 - x1)))
}

/// Performs quadratic interpolation for the point `x` given vectors of abscissas `xs` and ordinates `ys`.
///
/// # Panics
///
/// Panics when the point arrays are missing, incompatible, non-finite, not
/// strictly increasing, or numerically unstable. Runtime input boundaries
/// should prefer [`try_quadratic_interpolation`].
#[must_use]
pub fn quadratic_interpolation(x: f64, xs: &[f64], ys: &[f64]) -> f64 {
    try_quadratic_interpolation(x, xs, ys).unwrap_or_else(|error| panic!("{error}"))
}

/// Performs checked quadratic interpolation for `x`.
///
/// # Errors
///
/// Returns an [`InterpolationError`] when the point arrays are missing,
/// incompatible, non-finite, not strictly increasing, or numerically unstable.
pub fn try_quadratic_interpolation(
    x: f64,
    xs: &[f64],
    ys: &[f64],
) -> Result<f64, InterpolationError> {
    let n_elem = xs.len();
    let epsilon = 1e-8;

    validate_interpolation_inputs(x, xs, ys)?;

    let first_x = interpolation_point(xs, 0)?;
    let first_y = interpolation_point(ys, 0)?;
    let last_index = n_elem - 1;
    let last_x = interpolation_point(xs, last_index)?;
    let last_y = interpolation_point(ys, last_index)?;

    if x <= first_x {
        return Ok(first_y);
    }

    if x >= last_x {
        return Ok(last_y);
    }

    let pos = pos_search(x, xs);

    if (interpolation_point(xs, pos)? - x).abs() < epsilon {
        return interpolation_point(ys, pos);
    }

    if pos == 0 {
        return try_quad_polynomial(
            x,
            interpolation_point(xs, 0)?,
            interpolation_point(xs, 1)?,
            interpolation_point(xs, 2)?,
            interpolation_point(ys, 0)?,
            interpolation_point(ys, 1)?,
            interpolation_point(ys, 2)?,
        );
    }

    if pos == n_elem - 2 {
        return try_quad_polynomial(
            x,
            interpolation_point(xs, n_elem - 3)?,
            interpolation_point(xs, n_elem - 2)?,
            interpolation_point(xs, last_index)?,
            interpolation_point(ys, n_elem - 3)?,
            interpolation_point(ys, n_elem - 2)?,
            interpolation_point(ys, last_index)?,
        );
    }

    let w = try_linear_weight(
        interpolation_point(xs, pos)?,
        interpolation_point(xs, pos + 1)?,
        x,
    )?;

    Ok(linear_weighting(
        try_quad_polynomial(
            x,
            interpolation_point(xs, pos - 1)?,
            interpolation_point(xs, pos)?,
            interpolation_point(xs, pos + 1)?,
            interpolation_point(ys, pos - 1)?,
            interpolation_point(ys, pos)?,
            interpolation_point(ys, pos + 1)?,
        )?,
        try_quad_polynomial(
            x,
            interpolation_point(xs, pos)?,
            interpolation_point(xs, pos + 1)?,
            interpolation_point(xs, pos + 2)?,
            interpolation_point(ys, pos)?,
            interpolation_point(ys, pos + 1)?,
            interpolation_point(ys, pos + 2)?,
        )?,
        w,
    ))
}

fn validate_interpolation_inputs(x: f64, xs: &[f64], ys: &[f64]) -> Result<(), InterpolationError> {
    let n_elem = xs.len();
    if n_elem < 3 {
        return Err(InterpolationError::InsufficientPoints {
            minimum: 3,
            actual: n_elem,
        });
    }
    if n_elem != ys.len() {
        return Err(InterpolationError::LengthMismatch {
            xs_len: n_elem,
            ys_len: ys.len(),
        });
    }
    if !x.is_finite() {
        return Err(InterpolationError::NonFiniteInput {
            parameter: "x",
            index: None,
        });
    }
    for (index, value) in xs.iter().enumerate() {
        if !value.is_finite() {
            return Err(InterpolationError::NonFiniteInput {
                parameter: "xs",
                index: Some(index),
            });
        }
    }
    for (index, value) in ys.iter().enumerate() {
        if !value.is_finite() {
            return Err(InterpolationError::NonFiniteInput {
                parameter: "ys",
                index: Some(index),
            });
        }
    }
    for (left_index, pair) in xs.windows(2).enumerate() {
        let [left, right] = pair else {
            continue;
        };
        if right <= left {
            return Err(InterpolationError::UnsortedAbscissas {
                left_index,
                right_index: left_index + 1,
                left: *left,
                right: *right,
            });
        }
        let diff = right - left;
        if diff < f64::EPSILON * 2.0 {
            return Err(InterpolationError::AbscissasTooClose {
                left: *left,
                right: *right,
                diff,
                minimum: f64::EPSILON * 2.0,
            });
        }
    }

    Ok(())
}

fn interpolation_point(values: &[f64], index: usize) -> Result<f64, InterpolationError> {
    values
        .get(index)
        .copied()
        .ok_or(InterpolationError::InvalidIndex {
            index,
            len: values.len(),
        })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rstest::*;

    use super::*;

    #[rstest]
    #[case(0.0, 10.0, 5.0, 0.5)]
    #[case(1.0, 3.0, 2.0, 0.5)]
    #[case(0.0, 1.0, 0.25, 0.25)]
    #[case(0.0, 1.0, 0.75, 0.75)]
    fn test_linear_weight_valid_cases(
        #[case] x1: f64,
        #[case] x2: f64,
        #[case] x: f64,
        #[case] expected: f64,
    ) {
        let result = linear_weight(x1, x2, x);
        assert!(
            approx_eq!(f64, result, expected, epsilon = 1e-10),
            "Expected {expected}, was {result}"
        );
    }

    #[rstest]
    #[should_panic(expected = "too close for stable interpolation")]
    fn test_linear_weight_zero_divisor() {
        let _ = linear_weight(1.0, 1.0, 0.5);
    }

    #[rstest]
    #[should_panic(expected = "too close for stable interpolation")]
    fn test_linear_weight_near_equal_values() {
        // Values within machine epsilon should be rejected
        let _ = linear_weight(1.0, 1.0 + f64::EPSILON, 0.5);
    }

    #[rstest]
    fn test_linear_weight_with_small_differences() {
        // High-resolution data (e.g., nanosecond timestamps as seconds) should work
        let result = linear_weight(0.0, 1e-12, 5e-13);
        assert!(result.is_finite());
        assert!((result - 0.5).abs() < 1e-10); // Should be approximately 0.5
    }

    #[rstest]
    fn test_linear_weight_just_above_epsilon() {
        // Values differing by more than machine epsilon should work
        let result = linear_weight(1.0, 1.0 + 1e-9, 1.0 + 5e-10);
        // Should not panic and return a reasonable value
        assert!(result.is_finite());
    }

    #[rstest]
    #[case(1.0, 3.0, 0.5, 2.0)]
    #[case(10.0, 20.0, 0.25, 12.5)]
    #[case(0.0, 10.0, 0.0, 0.0)]
    #[case(0.0, 10.0, 1.0, 10.0)]
    fn test_linear_weighting(
        #[case] y1: f64,
        #[case] y2: f64,
        #[case] weight: f64,
        #[case] expected: f64,
    ) {
        let result = linear_weighting(y1, y2, weight);
        assert!(
            approx_eq!(f64, result, expected, epsilon = 1e-10),
            "Expected {expected}, was {result}"
        );
    }

    #[rstest]
    #[case(5.0, &[1.0, 2.0, 3.0, 4.0, 6.0, 7.0], 3)]
    #[case(1.5, &[1.0, 2.0, 3.0, 4.0], 0)]
    #[case(0.5, &[1.0, 2.0, 3.0, 4.0], 0)]
    #[case(4.5, &[1.0, 2.0, 3.0, 4.0], 3)]
    #[case(2.0, &[1.0, 2.0, 3.0, 4.0], 0)]
    fn test_pos_search(#[case] x: f64, #[case] xs: &[f64], #[case] expected: usize) {
        let result = pos_search(x, xs);
        assert_eq!(result, expected);
    }

    #[rstest]
    fn test_pos_search_edge_cases() {
        // Single element array
        let result = pos_search(5.0, &[10.0]);
        assert_eq!(result, 0);

        // Value at exact boundary
        let result = pos_search(3.0, &[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(result, 1); // Index of largest element < 3.0 is index 1 (value 2.0)

        // Two element array
        let result = pos_search(1.5, &[1.0, 2.0]);
        assert_eq!(result, 0);
    }

    #[rstest]
    fn test_pos_search_empty_slice() {
        let empty: &[f64] = &[];
        assert_eq!(pos_search(42.0, empty), 0);
    }

    #[rstest]
    fn test_quad_polynomial_linear_case() {
        // Test with three collinear points - should behave like linear interpolation
        let result = quad_polynomial(1.5, 1.0, 2.0, 3.0, 1.0, 2.0, 3.0);
        assert!(approx_eq!(f64, result, 1.5, epsilon = 1e-10));
    }

    #[rstest]
    fn test_quad_polynomial_parabola() {
        // Test with a simple parabola y = x^2
        // Points: (0,0), (1,1), (2,4)
        let result = quad_polynomial(1.5, 0.0, 1.0, 2.0, 0.0, 1.0, 4.0);
        let expected = 1.5 * 1.5; // Should be 2.25
        assert!(approx_eq!(f64, result, expected, epsilon = 1e-10));
    }

    #[rstest]
    #[should_panic(expected = "too close for stable interpolation")]
    fn test_quad_polynomial_duplicate_x() {
        let _ = quad_polynomial(0.5, 1.0, 1.0, 2.0, 0.0, 1.0, 4.0);
    }

    #[rstest]
    #[should_panic(expected = "too close for stable interpolation")]
    fn test_quad_polynomial_near_equal_x_values() {
        // x0 and x1 differ by only machine epsilon
        let _ = quad_polynomial(0.5, 1.0, 1.0 + f64::EPSILON, 2.0, 0.0, 1.0, 4.0);
    }

    #[rstest]
    fn test_quad_polynomial_with_small_differences() {
        // High-resolution data should work (e.g., 1e-12 spacing)
        let result = quad_polynomial(5e-13, 0.0, 1e-12, 2e-12, 0.0, 1.0, 4.0);
        assert!(result.is_finite());
    }

    #[rstest]
    fn test_quad_polynomial_just_above_epsilon() {
        // Values differing by more than machine epsilon should work
        let result = quad_polynomial(0.5, 0.0, 1.0 + 1e-9, 2.0, 0.0, 1.0, 4.0);
        // Should not panic and return a reasonable value
        assert!(result.is_finite());
    }

    #[rstest]
    #[expect(
        clippy::float_cmp,
        reason = "boundary inputs must return the exact boundary ys value"
    )]
    fn test_quadratic_interpolation_boundary_conditions() {
        let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = vec![1.0, 4.0, 9.0, 16.0, 25.0]; // y = x^2

        // Test below minimum
        let result = quadratic_interpolation(0.5, &xs, &ys);
        assert_eq!(result, ys[0]);

        // Test above maximum
        let result = quadratic_interpolation(6.0, &xs, &ys);
        assert_eq!(result, ys[4]);
    }

    #[rstest]
    fn test_quadratic_interpolation_exact_points() {
        let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = vec![1.0, 4.0, 9.0, 16.0, 25.0];

        // Test exact points
        for (i, &x) in xs.iter().enumerate() {
            let result = quadratic_interpolation(x, &xs, &ys);
            assert!(approx_eq!(f64, result, ys[i], epsilon = 1e-6));
        }
    }

    #[rstest]
    fn test_quadratic_interpolation_intermediate_values() {
        let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = vec![1.0, 4.0, 9.0, 16.0, 25.0]; // y = x^2

        // Test interpolation between points
        let result = quadratic_interpolation(2.5, &xs, &ys);
        let expected = 2.5 * 2.5; // Should be close to 6.25
        assert!((result - expected).abs() < 0.1); // Allow some interpolation error
    }

    #[rstest]
    #[should_panic(expected = "Need at least 3 points")]
    fn test_quadratic_interpolation_insufficient_points() {
        let xs = vec![1.0, 2.0];
        let ys = vec![1.0, 4.0];
        let _ = quadratic_interpolation(1.5, &xs, &ys);
    }

    #[rstest]
    #[should_panic(expected = "xs and ys must have the same length")]
    fn test_quadratic_interpolation_mismatched_lengths() {
        let xs = vec![1.0, 2.0, 3.0];
        let ys = vec![1.0, 4.0];
        let _ = quadratic_interpolation(1.5, &xs, &ys);
    }

    #[rstest]
    #[case(
        f64::NAN,
        &[1.0, 2.0, 3.0],
        &[1.0, 4.0, 9.0],
        InterpolationError::NonFiniteInput {
            parameter: "x",
            index: None,
        }
    )]
    #[case(
        1.5,
        &[1.0, f64::INFINITY, 3.0],
        &[1.0, 4.0, 9.0],
        InterpolationError::NonFiniteInput {
            parameter: "xs",
            index: Some(1),
        }
    )]
    #[case(
        1.5,
        &[1.0, 2.0, 3.0],
        &[1.0, f64::NAN, 9.0],
        InterpolationError::NonFiniteInput {
            parameter: "ys",
            index: Some(1),
        }
    )]
    fn test_try_quadratic_interpolation_rejects_non_finite_inputs(
        #[case] x: f64,
        #[case] xs: &[f64],
        #[case] ys: &[f64],
        #[case] expected: InterpolationError,
    ) {
        assert_eq!(try_quadratic_interpolation(x, xs, ys), Err(expected));
    }

    #[rstest]
    fn test_try_quadratic_interpolation_rejects_missing_points() {
        let result = try_quadratic_interpolation(1.5, &[1.0, 2.0], &[1.0, 4.0]);

        assert_eq!(
            result,
            Err(InterpolationError::InsufficientPoints {
                minimum: 3,
                actual: 2,
            })
        );
    }

    #[rstest]
    fn test_try_quadratic_interpolation_rejects_incompatible_lengths() {
        let result = try_quadratic_interpolation(1.5, &[1.0, 2.0, 3.0], &[1.0, 4.0]);

        assert_eq!(
            result,
            Err(InterpolationError::LengthMismatch {
                xs_len: 3,
                ys_len: 2,
            })
        );
    }

    #[rstest]
    #[case(&[1.0, 3.0, 2.0])]
    #[case(&[1.0, 2.0, 2.0])]
    fn test_try_quadratic_interpolation_rejects_unsorted_points(#[case] xs: &[f64]) {
        let result = try_quadratic_interpolation(1.5, xs, &[1.0, 4.0, 9.0]);

        assert!(matches!(
            result,
            Err(InterpolationError::UnsortedAbscissas {
                left_index: 1,
                right_index: 2,
                ..
            })
        ));
    }

    #[rstest]
    fn test_try_quadratic_interpolation_rejects_numerically_unstable_points() {
        let result =
            try_quadratic_interpolation(1.5, &[1.0, 1.0 + f64::EPSILON, 2.0], &[1.0, 4.0, 9.0]);

        assert!(matches!(
            result,
            Err(InterpolationError::AbscissasTooClose { .. })
        ));
    }

    proptest! {
        #[test]
        fn prop_try_quadratic_interpolation_returns_finite_for_valid_points(
            x in -1_000.0f64..1_000.0,
            base in -1_000.0f64..1_000.0,
            gap_1 in 0.001f64..100.0,
            gap_2 in 0.001f64..100.0,
            y_0 in -1_000.0f64..1_000.0,
            y_1 in -1_000.0f64..1_000.0,
            y_2 in -1_000.0f64..1_000.0,
        ) {
            let xs = [base, base + gap_1, base + gap_1 + gap_2];
            let ys = [y_0, y_1, y_2];

            let result = try_quadratic_interpolation(x, &xs, &ys);
            prop_assert!(matches!(result, Ok(value) if value.is_finite()));
        }
    }

    #[rstest]
    #[case(f64::NAN, 0.0, 1.0)]
    #[case(0.0, f64::NAN, 1.0)]
    #[case(0.0, 1.0, f64::NAN)]
    #[case(f64::INFINITY, 0.0, 1.0)]
    #[case(0.0, f64::NEG_INFINITY, 1.0)]
    #[should_panic(expected = "All inputs must be finite")]
    fn test_linear_weight_non_finite_panics(#[case] x1: f64, #[case] x2: f64, #[case] x: f64) {
        let _ = linear_weight(x1, x2, x);
    }

    #[rstest]
    #[should_panic(expected = "All inputs must be finite")]
    fn test_quad_polynomial_nan_panics() {
        let _ = quad_polynomial(f64::NAN, 0.0, 1.0, 2.0, 0.0, 1.0, 4.0);
    }

    #[rstest]
    #[should_panic(expected = "All inputs must be finite")]
    fn test_quad_polynomial_infinity_panics() {
        let _ = quad_polynomial(0.5, f64::INFINITY, 1.0, 2.0, 0.0, 1.0, 4.0);
    }
}
