use authentication_rs_lib::AuthenticationOpaque;
use curve25519_dalek::ristretto::RistrettoPoint;
use opaque_ke::{keypair::{Key, KeyPair}, rand::rngs::OsRng, ServerRegistration, RegistrationRequest, RegistrationUpload, ClientRegistration, ClientRegistrationFinishParameters};

pub fn start(server_public_key: &Key, registration_request_bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    let mut server_rng = OsRng;

    let server_registration_start_result = ServerRegistration::<AuthenticationOpaque>::start(
        &mut server_rng,
        RegistrationRequest::deserialize(registration_request_bytes).map_err(|err| format!("{}",err))?,
        server_public_key,
    )
    .map_err(|err| format!("{}",err))?;

    Ok((
        server_registration_start_result.state.serialize(),
        server_registration_start_result.message.serialize(),
    ))
}

pub fn finish(server_state_bytes: &[u8], registration_final_message_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let server_state =
        ServerRegistration::<AuthenticationOpaque>::deserialize(server_state_bytes).map_err(|err| format!("{}",err))?;
    let password_file = server_state
        .finish(RegistrationUpload::deserialize(registration_final_message_bytes).map_err(|err| format!("{}",err))?)
        .map_err(|err| format!("{}",err))?;
    Ok(password_file.serialize())
}

pub fn generate_dummy_password_file(key_pair: &KeyPair<RistrettoPoint>, username: &str) -> worker::Result<Vec<u8>>
{
    let mut rng = OsRng;
    let client_registration_start_result = ClientRegistration::<AuthenticationOpaque>::start(&mut rng, b"123").map_err(|err| format!("{}",err))?;
    
    let server_registration_start_result = ServerRegistration::<AuthenticationOpaque>::start(
        &mut rng,
        client_registration_start_result.message,
        key_pair.public(),
    )
    .map_err(|err| format!("{}",err))?;

    let client_finish_registration_result = client_registration_start_result.state.finish(
        &mut rng,
        server_registration_start_result.message,
        ClientRegistrationFinishParameters::WithIdentifiers(username.as_bytes().to_vec(), vec![]),
    ).map_err(|err| format!("{}",err))?;

    let server_finish_registration_result = server_registration_start_result.state.finish(
        client_finish_registration_result.message,
    ).map_err(|err| format!("{}",err))?;

    Ok(server_finish_registration_result.serialize())
}
