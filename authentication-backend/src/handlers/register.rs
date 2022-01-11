use std::collections::HashMap;

use opaque_ke::keypair::KeyPair;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{data::{AuthenticationData, profile::{ProfileData, UserProfile}, register::RegistrationData}, utils::{unwrap_abort, unwrap_res_abort}};

#[derive(Deserialize)]
struct HttpRegistrationRequest {
    username: String,
    mail: String,
    request: String,
}

#[derive(Serialize)]
struct ValidationErrors {
    message: String,
    fields: HashMap<String, String>,
}

fn validate_request(req: &HttpRegistrationRequest) -> Option<worker::Result<worker::Response>> {
    
    let mut request_valid = true;
    let mut errors_hashmap: HashMap<String, String> = HashMap::new();

    if !crate::utils::validate_username(&req.username) {
        request_valid = false;
        errors_hashmap.insert("username".to_string(), "Invalid username, please only use letters, numbers, underscores, dashes, and periods. Usernames must be between 3 and 15 characters long.".to_string());
    }

    let email_re = unwrap_res_abort(Regex::new(r#"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"#));

    if req.mail.len() < 5 || req.mail.len() > 100 || !email_re.is_match(&req.mail) {
        request_valid = false;
        errors_hashmap.insert("mail".to_string(), "Invalid email address, please enter a valid email address.".to_string());
    }

    if req.request.len() < 5 || req.request.len() > 256  {
        request_valid = false;
        errors_hashmap.insert("request".to_string(), "Invalid request payload.".to_string());
    }
    
    if request_valid {
        return None;
    }
    
    let validation_errors = ValidationErrors {
        message: "Some field(s) have invalid data".to_string(),
        fields: errors_hashmap
    };

    
    Some(worker::Response::error(serde_json::to_string(&validation_errors).unwrap_or(validation_errors.message), 400))
}

pub async fn start_handler<D>(mut req: worker::Request, ctx: worker::RouteContext<D>) -> worker::Result<worker::Response>
{
    match req.json::<HttpRegistrationRequest>().await {
        Ok(values) => {
            if let Some(err) = validate_request(&values) {
                return err;
            }

            let keypair_bytes =
                unwrap_res_abort(base64::decode(ctx.secret("SERVER_KEYPAIR")?.to_string()));

            let data = AuthenticationData::new(&ctx);

            if data.profile_already_registered(&values.username).await? {
                return worker::Response::error("User already registered", 400);
            }

            if data.profile_already_registered_waiting_mail_confirm(&values.username).await? {
                return worker::Response::error("User already registered, missing confirming email", 400);
            }

            if data.too_many_registration_attempts(&values.username).await? {
                return worker::Response::error("Too many registration retries", 400);
            }
            
            let key_pair =
                unwrap_res_abort(
                    KeyPair::<curve25519_dalek::ristretto::RistrettoPoint>::from_private_key_slice(
                    keypair_bytes.as_slice(),
                ));

            let (state, response) = crate::opaque::register::start(key_pair.public(), &base64::decode(values.request).map_err(|err| format!("{}",err))?)?;


            data.set_registration_state(&values.username, state).await?;

            worker::Response::ok(base64::encode(&response))
        }
        Err(ref e) =>
            worker::Response::error(format!("{}", e), 400)
    }
}

pub async fn finish_handler<D>(mut req: worker::Request, ctx: worker::RouteContext<D>) -> worker::Result<worker::Response>
{
    match req.json::<HttpRegistrationRequest>().await {
        Ok(values) => {

            if let Some(err) = validate_request(&values) {
                return err;
            }

            let data = AuthenticationData::new(&ctx);

            if data.profile_already_registered(&values.username).await? {
                return worker::Response::error("User already registered", 400);
            }

            if data.profile_already_registered_waiting_mail_confirm(&values.username).await? {
                return worker::Response::error("User already registered, missing confirming email", 400);
            }

            let state = data.get_registration_state(&values.username).await?;

            if state.is_none() {
                return worker::Response::error("No registration state", 400);
            }
            let state = unwrap_abort(state);

            let password_file =
                crate::opaque::register::finish(&state, &base64::decode(values.request).map_err(|err| format!("{}",err))?);
            
            data.remove_registration_state(&values.username).await?;

            let password_file = password_file?;

            let email_verification_key = crate::confirmation_email::send(&values.username, &values.mail, ctx.secret("EMAILER_KEY")?.to_string()).await?;

            let profile = UserProfile {
                username: values.username.to_string(),
                mail: values.mail,
                password_file: base64::encode(password_file),
                email_verification: email_verification_key,
            };
            
            data.save_profile(&values.username, &profile, 0, false, false).await?;

            worker::Response::ok("")
        }
        Err(e) =>
            worker::Response::error(format!("{}", e), 400)
    }
}

pub async fn confirm_mail_handler<D>(req: worker::Request, ctx: worker::RouteContext<D>) -> worker::Result<worker::Response>
{
    if let Some(username) = ctx.param("username") {
        let email_key = unwrap_res_abort(req.url()).query_pairs().find(|(key, _)| key == "k").map_or(String::new(), |(_, value)| value.to_string());
        let email_key_result = base64::decode_config(&email_key, base64::URL_SAFE);
        if crate::utils::validate_username(username) && email_key_result.is_ok() {
            let data = AuthenticationData::new(&ctx);
            let pending_user_profile = data.get_profile(username).await?;
            if let Some((profile, meta)) = pending_user_profile {
                if meta.e {
                    return worker::Response::error("Email already verified", 200);
                }
                if meta.l {
                    return worker::Response::error("Account locked", 401);
                }

                if profile.email_verification == email_key {
                    data.save_profile(username, &profile, 0, false, true).await?;
                    return worker::Response::ok("");
                }
            } 
        }
    }
    
    worker::Response::error("Bad request", 400)
}