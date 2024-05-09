use clap::Args;
use color_eyre::eyre::ErrReport;
use zeroize::Zeroize;

use crate::{otp::otp_element::OTPDatabase, utils};

use super::SubcommandExecutor;

#[derive(Args)]
pub struct PasswdArgs;

impl SubcommandExecutor for PasswdArgs {
    fn run_command(self, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let mut new_password = utils::verified_password("New password: ", 8);
        database
            .save_with_pw(&new_password)
            .map_err(ErrReport::from)?;
        new_password.zeroize();
        Ok(database)
    }
}
