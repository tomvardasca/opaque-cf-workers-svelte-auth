use opaque_ke::keypair::KeyPair;
use serde::Deserialize;
use worker::console_log;

use crate::{data::{AuthenticationData, login::LoginData, profile::{ProfileData, UserProfileMetadata}}, utils::{unwrap_abort}};

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    request: String,
}

pub async fn start_handler<D>(mut req: worker::Request, ctx: worker::RouteContext<D>) -> worker::Result<worker::Response>
{
    if let Ok(values) = req.json::<LoginRequest>().await {

        if !crate::utils::validate_username(&values.username) {
            return worker::Response::error("Invalid username", 400);
        }

        let data = AuthenticationData::new(&ctx);
        
        if data.too_many_login_attempts(&values.username).await?  {           
            return worker::Response::error("Too many login retries", 400);
        }

        if data.too_many_login_sessions_attempts(&values.username).await?  {           
            return worker::Response::error("Too many login session tries", 400);
        }

        let keypair_bytes =
            base64::decode(ctx.secret("SERVER_KEYPAIR")?.to_string()).map_err(|err| format!("{}",err))?;

        let key_pair =
            KeyPair::<curve25519_dalek::ristretto::RistrettoPoint>::from_private_key_slice(
                keypair_bytes.as_slice(),
            )
            .map_err(|err| format!("{}",err))?;

        let dummy_password_file = crate::opaque::register::generate_dummy_password_file(&key_pair, &values.username)?;
        
        let profile = data.get_profile(&values.username).await?;
        console_log!("profile: {:?}", profile);
        
        let password_file;
        let password_file_metadata;

        if let Some((profile, metadata)) = profile {
            password_file = base64::decode(profile.password_file).map_err(|err| format!("{}",err))?;
            password_file_metadata = metadata;
        } else {
            password_file_metadata = UserProfileMetadata { v: 0, l: true, e: false };
            password_file = dummy_password_file;
        }

        let (state, response) = crate::opaque::login::start(
            key_pair.private(),
            &base64::decode(values.request).map_err(|err| format!("{}",err))?,
            &password_file,
            &values.username,
            password_file_metadata.v,
        )?;

        data.set_login_state(&values.username, state).await?;

        return worker::Response::ok(base64::encode(&response));
    }
    worker::Response::error("Bad Request", 400)
}

pub async fn finish_handler<D>(mut req: worker::Request, ctx: worker::RouteContext<D>) -> worker::Result<worker::Response>
{
    if let Ok(values) = req.json::<LoginRequest>().await {
        if !crate::utils::validate_username(&values.username) {
            return worker::Response::error("Invalid username", 400);
        }
        
        let data = AuthenticationData::new(&ctx);

        let state = data.get_login_state(&values.username).await?;

        if state.is_none() {
            return worker::Response::error("No login state", 400);
        }



        let profile = data.get_profile(&values.username).await?;
        if profile.is_none() {
            return worker::Response::error("Username does not exist", 400);
        }

        let (_, metadata) = unwrap_abort(profile);
        if !metadata.e {
            return worker::Response::error("Email not verified", 403);
        }
        if metadata.l {
            return worker::Response::error("Account locked", 401);
        }

        let session_key = crate::opaque::login::finish( 
                    &unwrap_abort(state), 
                    &base64::decode(values.request).map_err(|err| format!("{}",err))?);

        data.remove_login_state(&values.username).await?;

        let session_key = session_key?;

        data.set_login_session(&values.username, &session_key).await?;
        return worker::Response::ok(base64::encode(&session_key));
    }
    worker::Response::error("Bad Request", 400)
}