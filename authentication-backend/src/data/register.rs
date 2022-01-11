use async_trait::async_trait;
use worker::Date;

use super::AuthenticationData;

const REGISTRATION_STATE_PREFIX: &str = "REGISTRATION_STATE";


#[async_trait(?Send)]
pub trait RegistrationData {
    async fn too_many_registration_attempts(&self, username: &str) -> worker::Result<bool>;
    async fn set_registration_state(&self, username: &str, state: Vec<u8>) -> worker::Result<()>;
    async fn get_registration_state(&self, username: &str) -> worker::Result<Option<Vec<u8>>>;
    async fn remove_registration_state(&self, username: &str) -> worker::Result<()>;
}

#[async_trait(?Send)]
impl  RegistrationData for AuthenticationData {
    async fn too_many_registration_attempts(&self, username: &str) -> worker::Result<bool> {
        let state = self.kv.get_with_metadata::<u64>(&format!("{}:{}", REGISTRATION_STATE_PREFIX, username)).await?;
        if let Some((_, creation_time)) = state {
            Ok((creation_time + 15 * 1000) > Date::now().as_millis())
        } else {
            Ok(false)
        }
    }

    async fn set_registration_state(&self, username: &str, state: Vec<u8>) -> worker::Result<()> {
        self.kv.put(&format!("{}:{}", REGISTRATION_STATE_PREFIX, username), base64::encode(state))?.metadata( Date::now().as_millis())?.expiration_ttl(60).execute().await?;
        Ok(())
    }

    async fn get_registration_state(&self, username: &str) -> worker::Result<Option<Vec<u8>>> {
        let state = self.kv.get_with_metadata::<u64>(&format!("{}:{}", REGISTRATION_STATE_PREFIX, username)).await?;
        if let Some((state, _)) = state {
            let state = base64::decode(state.as_string()).map_err(|err| format!("{}",err))?;
            return Ok(Some(state))
        }
        Ok(None)
    }

    async fn remove_registration_state(&self, username: &str) -> worker::Result<()> {
        self.kv.delete(&format!("{}:{}", REGISTRATION_STATE_PREFIX, username)).await.map_err(std::convert::Into::into)
    }

}