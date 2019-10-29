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
use super::Password;
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
    debug!("Creating new password for user id {}", user_id);

    let salt = random_salt();
    let mut hash = new_hash();

    scrypt(password, &salt, &*PARAMS, &mut hash);

    trace!("Handing password model to consumer");
    let model = make_model(user_id, &hash, &salt);
    f(model)
}

pub fn check_password(record: &Password, password: &[u8]) -> bool {
    let params = ScryptParams::new(record.logn(), record.param_r(), record.param_p());
    let mut calculated = new_hash();

    // If the hash length ever changes we'll need to use a dynamically-allocated Vec instead.
    assert_eq!(
        record.hash().len(),
        calculated.as_ref().len(),
        "Hash length mismatch (stored vs runtime)",
    );

    debug!("Checking password validity");
    scrypt(password, record.salt(), &params, &mut calculated);
    fixed_time_eq(record.hash(), &calculated)
}
