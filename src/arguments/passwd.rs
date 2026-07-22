use clap::Args;
use zeroize::Zeroize;

use crate::{otp::otp_element::OTPDatabase, path::DATABASE_PATH, storage, utils};

use super::SubcommandExecutor;

#[derive(Args)]
pub struct PasswdArgs;

impl SubcommandExecutor for PasswdArgs {
    fn run_command(self, mut database: OTPDatabase) -> eyre::Result<OTPDatabase> {
        let mut new_password = utils::try_verified_password("New password: ", 8)?;
        // Saves with a key derived from the new password and clears the
        // modified flag, so main() will not save again with the old key
        storage::save_with_pw(&mut database, &new_password, DATABASE_PATH.get().unwrap())?;
        new_password.zeroize();
        Ok(database)
    }
}
