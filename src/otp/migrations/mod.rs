use super::otp_element::OTPDatabase;
struct Migration<'a> {
    to_version: u16, // Database version which we are migrating on
    migration_function: &'a dyn Fn(&mut OTPDatabase), // Function to execute the migration
}
const MIGRATIONS_LIST: [Migration; 1] = [Migration {
    to_version: 2,
    migration_function: &migrate_to_2,
}];

fn migrate_to_2(database: &mut OTPDatabase) {
    database.version = 2;
}

pub fn migrate(database: &mut OTPDatabase) {
    let mut binding = MIGRATIONS_LIST;
    let migrations = binding.as_mut();
    migrations.sort_unstable_by(|c1, c2| c1.to_version.cmp(&c2.to_version));
    for i in migrations {
        if database.version < i.to_version {
            // Do the migration
            (i.migration_function)(database);
        }
    }
}
