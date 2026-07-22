use super::otp_element::OTPDatabase;
struct Migration<'a> {
    to_version: u16, // Database version which we are migrating on
    migration_function: &'a dyn Fn(&mut OTPDatabase) -> color_eyre::Result<()>, // Function to execute the migration
}
/// Migrations must be kept sorted by ascending `to_version`; `migrate` relies
/// on this ordering and asserts it in debug builds.
const MIGRATIONS_LIST: [Migration; 1] = [Migration {
    to_version: 2,
    migration_function: &migrate_to_2,
}];

fn migrate_to_2(database: &mut OTPDatabase) -> color_eyre::Result<()> {
    database.version = 2;
    Ok(())
}

pub fn migrate(database: &mut OTPDatabase) -> color_eyre::Result<()> {
    debug_assert!(
        MIGRATIONS_LIST.is_sorted_by_key(|m| m.to_version),
        "MIGRATIONS_LIST must be sorted by to_version"
    );
    for migration in &MIGRATIONS_LIST {
        if database.version < migration.to_version {
            // Do the migration
            (migration.migration_function)(database)?;
        }
    }
    Ok(())
}
