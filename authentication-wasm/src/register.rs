use opaque_ke::{rand::rngs::OsRng, ClientRegistration, RegistrationResponse, ClientRegistrationFinishParameters};
use wasm_bindgen::prelude::*;

use authentication_rs_lib::AuthenticationOpaque;

use crate::{js_err};


#[wasm_bindgen]
pub struct RegistrationFinal;

#[wasm_bindgen]
pub struct Registration {
    state: ClientRegistration<AuthenticationOpaque>,
    server_request: String,
}

#[wasm_bindgen]
impl Registration {
    #[wasm_bindgen(constructor)]
    pub fn new(password: &str) -> Result<Registration, JsValue> {
        crate::utils::set_panic_hook();
        
        let mut client_rng = OsRng;
        let client_registration_start_result =
        js_err!(ClientRegistration::<AuthenticationOpaque>::start(&mut client_rng, password.as_bytes()))?;
        Ok(Registration { state: client_registration_start_result.state, server_request: base64::encode(client_registration_start_result.message.serialize()) })
    }

    #[must_use]
    #[wasm_bindgen(getter=serverRequest)]
    pub fn server_request(&self) -> String {
        self.server_request.clone()
    }
    pub fn finish(self, username: &str, server_response: &str) -> Result<String, JsValue> {
        let mut client_rng = OsRng;
        let server_response_bytes = js_err!(base64::decode(server_response))?;

        let client_finish_registration_result = js_err!(self.state.finish(
            &mut client_rng,
            js_err!(RegistrationResponse::deserialize(&server_response_bytes[..]))?,
            ClientRegistrationFinishParameters::WithIdentifiers(username.as_bytes().to_vec(), vec![]),
        ))?;
        Ok(base64::encode(client_finish_registration_result.message.serialize()))
    }
}
