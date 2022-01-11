use serde::{Serialize, Deserialize};
use worker::{kv::KvStore, Date, js_sys::Boolean};

const AUTHENTICATION_KV: &str = "AUTHENTICATION";

const REGISTRATION_STATE_PREFIX: &str = "REGISTRATION_STATE";
const LOGIN_STATE_PREFIX: &str = "LOGIN_STATE";
const PROFILE_PREFIX: &str = "PROFILE";
const PROFILE_PENDING_PREFIX: &str = "PROFILE_PENDING";
const LOGIN_SESSION_PREFIX: &str = "LOGIN_STATE";

#[derive(Serialize, Deserialize, Debug)]
pub struct UserProfile {
    pub username: String,
    pub mail: String,
    pub password_file: String,
    pub email_verification: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserProfileMetadata {
    // version
    pub v: u8,
    // email verified
    pub e: bool,
    // account locked
    pub l: bool,
}


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

    pub async fn username_already_registered(&self, username: &str) -> worker::Result<bool> {
        let user = self.kv.get(&format!("{}:{}", PROFILE_PENDING_PREFIX, username)).await?;
        Ok(user.is_some())
    }

    pub async fn username_already_registered_waiting_mail_confirm(&self, username: &str) -> worker::Result<bool> {
        let user = self.kv.get(&format!("{}:{}", PROFILE_PREFIX, username)).await?;
        Ok(user.is_some())
    }

    pub async fn too_many_registration_attempts(&self, username: &str) -> worker::Result<bool> {
        let state = self.kv.get_with_metadata::<u64>(&format!("{}:{}", REGISTRATION_STATE_PREFIX, username)).await?;
        if let Some((_, creation_time)) = state {
            Ok((creation_time + 60 * 1000) > Date::now().as_millis())
        } else {
            Ok(false)
        }
    }

    pub async fn set_registration_state(&self, username: &str, state: Vec<u8>) -> worker::Result<()> {
        self.kv.put(&format!("{}:{}", REGISTRATION_STATE_PREFIX, username), base64::encode(state))?.metadata( Date::now().as_millis())?.expiration_ttl(60).execute().await?;
        Ok(())
    }

    pub async fn get_registration_state(&self, username: &str) -> worker::Result<Option<Vec<u8>>> {
        let state = self.kv.get_with_metadata::<u64>(&format!("{}:{}", REGISTRATION_STATE_PREFIX, username)).await?;
        if let Some((state, _)) = state {
            let state = base64::decode(state.as_string()).map_err(|err| format!("{}",err))?;
            return Ok(Some(state))
        }
        Ok(None)
    }

    pub async fn remove_registration_state(&self, username: &str) -> worker::Result<()> {
        self.kv.delete(&format!("{}:{}", REGISTRATION_STATE_PREFIX, username)).await.map_err(|e| e.into())
    }

    pub async fn save_profile(&self, username: &str, profile: &UserProfile, version: u8, locked: bool, email_verified: bool) -> worker::Result<()> {
        self.kv.put(&format!("{}:{}", PROFILE_PREFIX, username), serde_json::to_string(profile).map_err(|err| format!("{}",err))?)?
        .metadata( UserProfileMetadata { v: version, l: locked, e: email_verified })?
        .execute().await?;
        Ok(())
    }

    pub async fn get_profile(&self, username: &str) -> worker::Result<Option<(UserProfile, UserProfileMetadata)>> {
        let profile = self.kv.get_with_metadata::<UserProfileMetadata>(&format!("{}:{}", PROFILE_PREFIX, username)).await?;
        if let Some((profile, metadata)) = profile {
            let profile = profile.as_json()?;
            return Ok(Some((profile, metadata)))
        }
        Ok(None)
    }

    pub async fn too_many_login_attempts(&self, username: &str) -> worker::Result<bool> {
        let state = self.kv.get_with_metadata::<u64>(&format!("{}:{}", LOGIN_STATE_PREFIX, username)).await?;
        if let Some((_, creation_time)) = state {
            Ok((creation_time + 60 * 1000) > Date::now().as_millis())
        } else {
            Ok(false)
        }
    }

    pub async fn set_login_state(&self, username: &str, state: Vec<u8>) -> worker::Result<()> {
        self.kv.put(&format!("{}:{}", LOGIN_STATE_PREFIX, username), base64::encode(state))?.metadata( Date::now().as_millis())?.expiration_ttl(60).execute().await?;
        Ok(())
    }

    pub async fn get_login_state(&self, username: &str) -> worker::Result<Option<Vec<u8>>> {
        let state = self.kv.get_with_metadata::<u64>(&format!("{}:{}", LOGIN_STATE_PREFIX, username)).await?;
        if let Some((state, _)) = state {
            let state = base64::decode(state.as_string()).map_err(|err| format!("{}",err))?;
            return Ok(Some(state))
        }
        Ok(None)
    }

    pub async fn remove_login_state(&self, username: &str) -> worker::Result<()> {
        self.kv.delete(&format!("{}:{}", LOGIN_STATE_PREFIX, username)).await.map_err(|e| e.into())
    }

    pub async fn set_login_session(&self, username: &str, session_key: &[u8]) -> worker::Result<()> {
        self.kv.put(&format!("{}:{}", LOGIN_SESSION_PREFIX, username), base64::encode(session_key))?.metadata( Date::now().as_millis())?.execute().await?;
        Ok(())
    }

    pub async fn too_many_login_sessions_attempts(&self, username: &str) -> worker::Result<bool> {
        let state = self.kv.get_with_metadata::<u64>(&format!("{}:{}", LOGIN_SESSION_PREFIX, username)).await?;
        if let Some((_, creation_time)) = state {
            Ok((creation_time + 60 * 1000) > Date::now().as_millis())
        } else {
            Ok(false)
        }
    }

    pub async fn get_login_session(&self, username: &str) -> worker::Result<Option<Vec<u8>>> {
        let state = self.kv.get_with_metadata::<u64>(&format!("{}:{}", LOGIN_SESSION_PREFIX, username)).await?;
        if let Some((state, _)) = state {
            let state = base64::decode(state.as_string()).map_err(|err| format!("{}",err))?;
            return Ok(Some(state))
        }
        Ok(None)
    }

    pub async fn remove_login_session(&self, username: &str) -> worker::Result<()> {
        self.kv.delete(&format!("{}:{}", LOGIN_SESSION_PREFIX, username)).await.map_err(|e| e.into())
    }

}