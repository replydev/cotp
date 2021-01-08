use crate::database_loader;
use crate::otp_helper;
use crate::importers;
use crate::cryptograpy::prompt_for_passwords;

pub fn help(){
    println!("USAGE:");
    println!("  cotp [SUBCOMMAND]");
    println!();
    println!("ARGUMENTS:");
    println!("  -a,--add [ISSUER] [LABEL]       | Add a new OTP code");
    println!("  -e,--edit [ID] [ISSUER] [LABEL] | Edit an OTP code");
    println!("  -r,--remove [ID]                         | Remove an OTP code");
    println!("  -i,--import [APPNAME] [PATH]             | Import a backup from a given application");
    println!("  -ex,--export                             | Export the entire database in a plaintext json format");
    println!("  -j,--json                                | Print results in json format");
    println!("  -s,--single                              | Print OTP codes in single mode");
    println!("  -h,--help                                | Print this help");
}

pub fn import(args: Vec<String>){
    if args.len() == 4{
        let result: Result<Vec<database_loader::OTPElement>,String>;
        let elements: Vec<database_loader::OTPElement>;

        match &args[2][..]{
            "cotp" | "andotp" => result = importers::and_otp::import(&args[3]),
            "aegis" => result = importers::aegis::import(&args[3]),
            _=> {
                println!("Invalid argument: {}", &args[2]);
                return;
            }
        }

        match result {
            Ok(result) => elements = result,
            Err(e) => {
                eprintln!("An error occurred: {}", e);
                return;
            }
        }
        
        match database_loader::overwrite_database(elements){
            Ok(()) => {
                println!("Successfully imported database");
            },
            Err(e) => {
                eprintln!("An error occurred during database overwriting: {}",e);
            }
        }
    }
    else{
        println!("Invalid arguments, type cotp --import [APPNAME] [PATH]");
        println!("cotp can import backup from:");
        println!("\"cotp\"");
        println!("\"aegis\"");
        println!("\"andotp\"");
    }
}

pub fn add(args: Vec<String>){
    if args.len() == 4{
        match database_loader::add_element(&prompt_for_passwords("Insert the secret: ",0),&args[2],&args[3]){
            Ok(()) => println!("Success"),
            Err(e) => eprintln!("An error occurred: {}",e)
        }
    }
    else{
        println!("Invalid arguments, type cotp --add [ISSUER] [LABEL]");
    }
}

pub fn remove(args: Vec<String>){
    if args.len() == 3{
        let id = args[2].parse::<usize>().unwrap();

        match database_loader::remove_element_from_db(id){
            Ok(()) => println!("Success"),
            Err(e) => eprintln!("An error has occurred: {}",e)
        }
    }
    else{
        println!("Invalid argument, type cotp --remove <index>");
    }
}

pub fn edit(args: Vec<String>){
    if args.len() == 5{
        let id = args[2].parse::<usize>().unwrap();
        let secret = &prompt_for_passwords("Inser the secret (type ENTER to skip modification): ",0);
        let issuer = &args[3];
        let label = &args[4];
        match database_loader::edit_element(id, &secret, &issuer, &label){
            Ok(()) => println!("Success"),
            Err(e) => eprintln!("An error occurred: {}",e)
        }
    }
    else{
        println!("Invalid arguments, type cotp --edit [ID] [ISSUER] [LABEL]\n\nReplace the attribute value with \".\" to skip the attribute modification");
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
                eprintln!("An error occured while exporting database: {}", e);
            }
        }
    }
    else{
        println!("Invalid argument, type cotp --export");
    }
}

pub fn json(args: Vec<String>){
    if args.len() == 2{
        match otp_helper::get_json_results(){
            Ok(results) => println!("{}",results),
            Err(e) => eprintln!("An error occurred while getting json result: {}",e),
        }
    }
    else{
        println!("Invalid argument, type cotp --json");
    }
}

pub fn single(args: Vec<String>){
    if args.len() == 2{
        match otp_helper::read_codes(){
            Ok(result) => {
                if result.len() == 0{
                    println!("No codes, type \"cotp -h\" to get help");
                }
                else{
                    otp_helper::show_codes(&result);
                }
            },
            Err(e) => eprintln!("An error occurred: {}",e)
        }
    }
    else{
       println!("Invalid argument, type cotp --single");
    }
 }