use super::SubcommandExecutor;
use crate::{otp::otp_element::OTPDatabase, utils};
use clap::Args;
use color_eyre::eyre::eyre;
use zeroize::Zeroizing;

#[derive(Args)]
pub struct PasswdArgs {
    /// Read the new password from this descriptor (Unix)
    #[arg(long)]
    pub new_password_fd: Option<u32>,
}

impl SubcommandExecutor for PasswdArgs {
    fn run_command(self, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let new_password = Zeroizing::new(if let Some(fd) = self.new_password_fd {
            utils::password_from_fd(fd)?
        } else {
            utils::verified_password("New password: ", 8)?
        });
        if new_password.chars().count() < 8 {
            return Err(eyre!("New password must contain at least 8 characters"));
        }
        let (key, salt) = database.save_with_pw(&new_password)?;
        let key = Zeroizing::new(key);
        crate::persistence::unlock(&key, &salt)?;
        Ok(database)
    }
}
