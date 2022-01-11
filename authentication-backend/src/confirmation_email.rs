use opaque_ke::rand::{rngs::OsRng, RngCore};
use serde_json::json;
use worker::Result;
pub async fn send(username: &str, _email: &str, emailer_key: &str) -> Result<String> {
     let mut email_verification_key = [0u8; 32];
     OsRng.fill_bytes(&mut email_verification_key);
     let email_verification_key = base64::encode_config(&email_verification_key, base64::URL_SAFE);

     let mut headers = worker::Headers::new();
     headers.append("Content-Type", "application/json")?;
     headers.append("Accept", "application/json")?;
     headers.append("api-key", emailer_key)?;
     let body = json!({
          "sender": {
               "name": "Tomé Vardasca",
               "email": "tome@vardas.ca"
          },
          "to": [
               {
                    "email": "tome@vardas.ca",
                    "name": "Tomé Vardasca"
               }
          ],
          "textContent": format!("Please confirm your email address by clicking on the link http://127.0.0.1:8787/register/confirm/{}?k={}", &username, &email_verification_key),
          "htmlContent": format!("<!DOCTYPE html> <html> <body> <h1>Confirm you email</h1> <p>Please confirm your email address by clicking on the link below</p> <a href=\"http://127.0.0.1:8787/register/confirm/{username}?k={key}\">http://127.0.0.1:8787/register/confirm/{username}?k={key}</a> </body> </html>", username = &username, key = &email_verification_key),
          "subject": "Login Email confirmation"
     });

     let mut req_init = worker::RequestInit::new();
     req_init.with_method(worker::Method::Post).with_headers(headers).with_body(Some(worker::wasm_bindgen::JsValue::from_str(&body.to_string())));

     let req = worker::Request::new_with_init("https://api.sendinblue.com/v3/smtp/email", &req_init)?;

     let fetch = worker::Fetch::Request(req);
     let mut response = fetch.send().await?;
     let text = response.text().await?;

     worker::console_log!("email verification : [ status: '{}', text: '{}' ]", response.status_code(), text);
     Ok(email_verification_key)
}