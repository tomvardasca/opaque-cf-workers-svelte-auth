
use opaque_ke::{rand::rngs::OsRng, ClientLogin, ClientLoginStartParameters, CredentialResponse, ClientLoginFinishParameters};
use wasm_bindgen::prelude::*;

use authentication_rs_lib::AuthenticationOpaque;

use crate::js_err;

#[wasm_bindgen]
pub struct LoginFinal {
    server_request: String,
    session_key: String,
}

#[wasm_bindgen]
impl LoginFinal {
    #[must_use]
    #[wasm_bindgen(getter=serverRequest)]
    pub fn server_request(&self) -> String {
        self.server_request.clone()
    }

    #[must_use]
    #[wasm_bindgen(getter=sessionKey)]
    pub fn session_key(&self) -> String {
        self.session_key.clone()
    }
}

#[wasm_bindgen]
pub struct Login {
    state: ClientLogin<AuthenticationOpaque>,
    server_request: String,
}

#[wasm_bindgen]
impl Login {
    #[wasm_bindgen(constructor)]
    pub fn new(username: &str, password: &str) -> Result<Login, JsValue> {
        crate::utils::set_panic_hook();

        let mut client_rng = OsRng;
        let client_login_start_result =
        js_err!(ClientLogin::<AuthenticationOpaque>::start(&mut client_rng, password.as_bytes(), ClientLoginStartParameters::WithInfo(username.as_bytes().to_vec())))?;
        Ok(Login { state: client_login_start_result.state, server_request: base64::encode(js_err!(client_login_start_result.message.serialize())?) })
    }


    #[must_use]
    #[wasm_bindgen(getter=serverRequest)]
    pub fn server_request(&self) -> String {
        self.server_request.clone()
    }

    pub fn finish(self, username: &str, server_response: &str) -> Result<LoginFinal, JsValue> {
        let server_response_bytes = js_err!(base64::decode(server_response))?;

        let client_finish_login_result = js_err!(self.state.finish(
            js_err!(CredentialResponse::deserialize(&server_response_bytes[..]))?,
            ClientLoginFinishParameters::WithIdentifiers(username.as_bytes().to_vec(), vec![]),
        ))?;
        Ok(LoginFinal{ server_request: base64::encode(js_err!(client_finish_login_result.message.serialize())?),
            session_key: base64::encode(client_finish_login_result.session_key)
        })
    }
}