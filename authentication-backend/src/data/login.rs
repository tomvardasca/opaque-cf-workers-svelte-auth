use async_trait::async_trait;
use worker::Date;

use super::AuthenticationData;

const LOGIN_STATE_PREFIX: &str = "LOGIN_STATE";

const LOGIN_SESSION_PREFIX: &str = "LOGIN_STATE";

#[async_trait(?Send)]
pub trait LoginData {
    async fn too_many_login_attempts(&self, username: &str) -> worker::Result<bool>;
    async fn set_login_state(&self, username: &str, state: Vec<u8>) -> worker::Result<()>;
    async fn get_login_state(&self, username: &str) -> worker::Result<Option<Vec<u8>>>;
    async fn remove_login_state(&self, username: &str) -> worker::Result<()>;
    async fn set_login_session(&self, username: &str, session_key: &[u8]) -> worker::Result<()>;
    async fn too_many_login_sessions_attempts(&self, username: &str) -> worker::Result<bool>;
    async fn get_login_session(&self, username: &str) -> worker::Result<Option<Vec<u8>>>;
    async fn remove_login_session(&self, username: &str) -> worker::Result<()>;
}

#[async_trait(?Send)]
impl LoginData for AuthenticationData {
    async fn too_many_login_attempts(&self, username: &str) -> worker::Result<bool> {
        let state = self.kv.get_with_metadata::<u64>(&format!("{}:{}", LOGIN_STATE_PREFIX, username)).await?;
        if let Some((_, creation_time)) = state {
            Ok((creation_time + 5 * 1000) > Date::now().as_millis())
        } else {
            Ok(false)
        }
    }

    async fn set_login_state(&self, username: &str, state: Vec<u8>) -> worker::Result<()> {
        self.kv.put(&format!("{}:{}", LOGIN_STATE_PREFIX, username), base64::encode(state))?.metadata( Date::now().as_millis())?.expiration_ttl(60).execute().await?;
        Ok(())
    }

    async fn get_login_state(&self, username: &str) -> worker::Result<Option<Vec<u8>>> {
        let state = self.kv.get_with_metadata::<u64>(&format!("{}:{}", LOGIN_STATE_PREFIX, username)).await?;
        if let Some((state, _)) = state {
            let state = base64::decode(state.as_string()).map_err(|err| format!("{}",err))?;
            return Ok(Some(state))
        }
        Ok(None)
    }

    async fn remove_login_state(&self, username: &str) -> worker::Result<()> {
        self.kv.delete(&format!("{}:{}", LOGIN_STATE_PREFIX, username)).await.map_err(std::convert::Into::into)
    }

    async fn set_login_session(&self, username: &str, session_key: &[u8]) -> worker::Result<()> {
        self.kv.put(&format!("{}:{}", LOGIN_SESSION_PREFIX, username), base64::encode(session_key))?.metadata( Date::now().as_millis())?.execute().await?;
        Ok(())
    }

    async fn too_many_login_sessions_attempts(&self, username: &str) -> worker::Result<bool> {
        let state = self.kv.get_with_metadata::<u64>(&format!("{}:{}", LOGIN_SESSION_PREFIX, username)).await?;
        if let Some((_, creation_time)) = state {
            Ok((creation_time + 5 * 1000) > Date::now().as_millis())
        } else {
            Ok(false)
        }
    }

    async fn get_login_session(&self, username: &str) -> worker::Result<Option<Vec<u8>>> {
        let state = self.kv.get_with_metadata::<u64>(&format!("{}:{}", LOGIN_SESSION_PREFIX, username)).await?;
        if let Some((state, _)) = state {
            let state = base64::decode(state.as_string()).map_err(|err| format!("{}",err))?;
            return Ok(Some(state))
        }
        Ok(None)
    }

    async fn remove_login_session(&self, username: &str) -> worker::Result<()> {
        self.kv.delete(&format!("{}:{}", LOGIN_SESSION_PREFIX, username)).await.map_err(std::convert::Into::into)
    }    
}