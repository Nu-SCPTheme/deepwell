/*
 * auth/crypto.rs
 *
 * deepwell - Database management and migrations service
 * Copyright (C) 2019 Ammon Smith
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

use super::models::*;
use crate::user::UserId;
use crate::Result;
use crypto::scrypt::{scrypt, ScryptParams};
use crypto::util::fixed_time_eq;
use rand::{rngs::OsRng, RngCore};

const PARAM_LOGN: u8 = 13;
const PARAM_R: u32 = 8;
const PARAM_P: u32 = 16;

type Hash = [u8; 32];
type Salt = [u8; 16];

lazy_static! {
    static ref PARAMS: ScryptParams = ScryptParams::new(PARAM_LOGN, PARAM_R, PARAM_P);
}

#[inline]
fn make_model<'a>(user_id: UserId, hash: &'a [u8], salt: &'a [u8]) -> NewPassword<'a> {
    NewPassword {
        user_id: user_id.into(),
        hash,
        salt,
        logn: PARAM_LOGN.into(),
        param_r: PARAM_R as i32,
        param_p: PARAM_P as i32,
    }
}

fn random_salt() -> Salt {
    let mut bytes = [0; 16];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

#[inline]
fn new_hash() -> Hash {
    [0; 32]
}

pub fn new_password<F>(user_id: UserId, password: &[u8], f: F) -> Result<()>
where
    F: FnOnce(NewPassword<'_>) -> Result<()>,
{
    let salt = random_salt();
    let mut hash = new_hash();

    scrypt(password, &salt, &*PARAMS, &mut hash);

    let model = make_model(user_id, &hash, &salt);
    f(model)
}
