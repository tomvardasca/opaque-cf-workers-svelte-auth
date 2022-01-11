use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use super::AuthenticationData;

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

const PROFILE_PREFIX: &str = "PROFILE";
const PROFILE_PENDING_PREFIX: &str = "PROFILE_PENDING";

#[async_trait(?Send)]
pub trait ProfileData {
    async fn profile_already_registered(&self, username: &str) -> worker::Result<bool>;
    async fn profile_already_registered_waiting_mail_confirm(&self, username: &str) -> worker::Result<bool>;
    async fn save_profile(&self, username: &str, profile: &UserProfile, version: u8, locked: bool, email_verified: bool) -> worker::Result<()>;
    async fn get_profile(&self, username: &str) -> worker::Result<Option<(UserProfile, UserProfileMetadata)>>;
}

#[async_trait(?Send)]
impl ProfileData for AuthenticationData {
    async fn get_profile(&self, username: &str) -> worker::Result<Option<(UserProfile, UserProfileMetadata)>> {
        let profile = self.kv.get_with_metadata::<UserProfileMetadata>(&format!("{}:{}", PROFILE_PREFIX, username)).await?;
        if let Some((profile, metadata)) = profile {
            let profile = profile.as_json()?;
            return Ok(Some((profile, metadata)))
        }
        Ok(None)
    }

    async fn profile_already_registered(&self, username: &str) -> worker::Result<bool> {
        let user = self.kv.get(&format!("{}:{}", PROFILE_PENDING_PREFIX, username)).await?;
        Ok(user.is_some())
    }

    async fn profile_already_registered_waiting_mail_confirm(&self, username: &str) -> worker::Result<bool> {
        let user = self.kv.get(&format!("{}:{}", PROFILE_PREFIX, username)).await?;
        Ok(user.is_some())
    }

    async fn save_profile(&self, username: &str, profile: &UserProfile, version: u8, locked: bool, email_verified: bool) -> worker::Result<()> {
        self.kv.put(&format!("{}:{}", PROFILE_PREFIX, username), serde_json::to_string(profile).map_err(|err| format!("{}",err))?)?
        .metadata( UserProfileMetadata { v: version, l: locked, e: email_verified })?
        .execute().await?;
        Ok(())
    }    
}