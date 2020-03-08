/*
 * math.rs
 *
 * deepwell-core - Database management and migrations service
 * Copyright (C) 2019-2020 Ammon Smith
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use std::f32::consts::{PI, SQRT_2};

/// Small enough value to stop evaluation at.
const EPSILON: f32 = 0.0001;

// Helper functions to improve readability

#[inline]
fn sqrt(x: f32) -> f32 {
    x.sqrt()
}

// Main math functions

/// Probit, or the normal quantile function.
pub fn probit(x: f32) -> f32 {
    let p = {
        if x == 0.0 {
            EPSILON
        } else if x >= 1.0 {
            1.0 - EPSILON
        } else {
            x
        }
    };

    SQRT_2 * inverse_error(2.0 * p - 1.0)
}

/// Inverse Guassian error function.
pub fn inverse_error(x: f32) -> f32 {
    let a = (8.0 * (PI - 3.0)) / (3.0 * PI * (4.0 - PI));
    let lnx2 = (1.0 - x * x).ln();
    let twopx = 2.0 / (PI * a) + lnx2 / 2.0;
    let inv = sqrt(sqrt(twopx.powi(2) - lnx2 / a) - twopx);

    inv.copysign(x)
}

// Tests
cfg_if! {
if #[cfg(test)] {
use crate::scoring::f32_eq;

const PRECISION: f32 = 0.000001;
}
}

#[test]
fn test_probit() {
    f32_eq(probit(0.0), -3.706124, PRECISION);
    f32_eq(probit(0.1), -1.281053, PRECISION);
    f32_eq(probit(0.2), -0.841549, PRECISION);
    f32_eq(probit(0.3), -0.524393, PRECISION);
    f32_eq(probit(0.4), -0.253346, PRECISION);
    f32_eq(probit(0.5), 0.0, PRECISION);
    f32_eq(probit(0.8), 0.8415493, PRECISION);
    f32_eq(probit(1.0), 3.706049, PRECISION);
    f32_eq(probit(1.3), 3.706049, PRECISION);
}

#[test]
fn test_inverse_error() {
    f32_eq(inverse_error(0.1), 0.088856, PRECISION);
    f32_eq(inverse_error(0.3), 0.272462, PRECISION);
    f32_eq(inverse_error(0.8), 0.905842, PRECISION);
    f32_eq(inverse_error(-0.1), -0.088856, PRECISION);
    f32_eq(inverse_error(-0.3), -0.272462, PRECISION);
    f32_eq(inverse_error(-0.5), -0.476919, PRECISION);
}
