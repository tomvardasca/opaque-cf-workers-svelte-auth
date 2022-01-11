use authentication_rs_lib::AuthenticationOpaque;
use opaque_ke::{keypair::Key, ServerRegistration, rand::rngs::OsRng, ServerLogin, CredentialRequest, ServerLoginStartParameters, CredentialFinalization};

use crate::utils::unwrap_res_abort;

pub fn start(
    server_key_private: &Key,
    credential_request_bytes: &[u8],
    password_file_bytes: &[u8],
    username: &str,
    _version: u8,
) -> Result<(Vec<u8>, Vec<u8>), String> {
    let password_file =
    unwrap_res_abort(ServerRegistration::<AuthenticationOpaque>::deserialize(password_file_bytes));
    let mut server_rng = OsRng;
    let server_login_start_result = ServerLogin::start(
        &mut server_rng,
        password_file,
        server_key_private,
        unwrap_res_abort(CredentialRequest::<AuthenticationOpaque>::deserialize(credential_request_bytes)),
        ServerLoginStartParameters::WithIdentifiers(username.as_bytes().to_vec(), vec![]),
    )
    .map_err(|err| format!("{}",err))?;

    Ok((
        server_login_start_result.state.serialize().map_err(|err| format!("{}",err))?,
        server_login_start_result.message.serialize().map_err(|err| format!("{}",err))?,
    ))
}

pub fn finish(server_login_state_bytes: &[u8], credential_finalization_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let server_login_state =
        ServerLogin::<AuthenticationOpaque>::deserialize(server_login_state_bytes).map_err(|err| format!("{}",err))?;
    let server_login_finish_result = server_login_state
        .finish(CredentialFinalization::deserialize(credential_finalization_bytes).map_err(|err| format!("{}",err))?)
        .map_err(|err| format!("{}",err))?;

    Ok(server_login_finish_result.session_key)
}
