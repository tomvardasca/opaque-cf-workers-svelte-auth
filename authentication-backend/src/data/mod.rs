pub mod register;
pub mod profile;
pub mod login;

use worker::{kv::KvStore};

use crate::utils::unwrap_res_abort;

const AUTHENTICATION_KV: &str = "AUTHENTICATION";

pub struct AuthenticationData{
    kv: KvStore,
}

impl<'a> AuthenticationData {
    pub fn new<D>(ctx: &'a worker::RouteContext<D>) -> Self {
        let kv = unwrap_res_abort(ctx.kv(AUTHENTICATION_KV));
        Self {
            kv,
        }
    }
}