use super::database_loader;
use super::otp_helper;
use crate::importers;

pub fn help(){
    println!("ARGUMENTS:");
    println!("-a,--add <secret> <issuer> <label>       | Add a new OTP code");
    println!("-r,--remove <secret> <issuer> <label>    | Remove an OTP code");
    println!("-e,--edit <id> <secret> <issuer> <label> | Edit an OTP code");
    println!("-i,--import aegis,andotp <filename>      | Import a backup from a given application");
    println!("-ex,--export                             | Export the entire database in a plaintext json format");
    println!("-j,--json                                | Print results in json format");
    println!("-h,--help                                | Print this help");
}

pub fn import(args: Vec<String>){
    if args.len() == 4{
        let result: Result<Vec<database_loader::OTPElement>,String>;
        let elements: Vec<database_loader::OTPElement>;

        match &args[2][..]{
            "andotp" => result = importers::and_otp::import(&args[3]),
            "aegis" => result = importers::aegis::import(&args[3]),
            _=> {
                println!("Invalid argument: {}", &args[2]);
                return;
            }
        }

        match result {
            Ok(result) => elements = result,
            Err(e) => {
                println!("An error occurred: {}", e);
                return;
            }
        }
        database_loader::overwrite_database(elements);
        println!("Successfully imported database");
    }
    else{
        println!("Invalid arguments, type cotp --import <backup_format> <path>");
    }
}

pub fn add(args: Vec<String>){
    if args.len() == 5{
        match database_loader::add_element(&args[2],&args[3],&args[4]){
            Ok(()) => println!("Success"),
            Err(e) => println!("An error occurred: {}",e)
        }
    }
    else{
        println!("Invalid arguments, type cotp --add <secret> <issuer> <label>");
    }
}

pub fn remove(args: Vec<String>){
    if args.len() == 3{
        let id = args[2].parse::<usize>().unwrap();

        match database_loader::remove_element_from_db(id){
            Ok(()) => println!("Success"),
            Err(e) => println!("An error has occurred: {}",e)
        }
    }
    else{
        println!("Invalid argument, type cotp --remove <index>");
    }
}

pub fn edit(args: Vec<String>){
    if args.len() == 6{
        let id = args[2].parse::<usize>().unwrap();
        let secret = &args[3];
        let issuer = &args[4];
        let label = &args[5];
        database_loader::edit_element(id, &secret, &issuer, &label).expect("An error occured");
    }
    else{
        println!("Invalid arguments, type cotp --edit <id> <secret> <issuer> <label>\n\nReplace the attribute value with \".\" to skip the attribute modification");
    }
}

pub fn export(args: Vec<String>){
    if args.len() == 2{
        let export_result = database_loader::export_database();
        match export_result{
            Ok(export_result) => {
                println!("Database was successfully exported at {}", export_result);
            },
            Err(e) =>{
                println!("An error as occured while exporting database: {}", e);
            }
        }
    }
    else{
        println!("Invalid argument, type cotp --export");
    }
}

pub fn json(args: Vec<String>){
    if args.len() == 2{
        println!("{}",otp_helper::get_json_results().expect("Failed to get json results"));
    }
    else{
        println!("Invalid argument, type cotp --json");
    }
}
