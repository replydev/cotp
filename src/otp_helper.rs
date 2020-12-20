use otp::make_totp;
use super::database_loader;


pub fn show_codes(){
    let elements: Vec<database_loader::OTPElement> = database_loader::read_from_file();
    for i in 0..elements.len() {
        print_totp(i,&elements[i]);
    }
}

fn print_totp(i: usize,element: &database_loader::OTPElement){
    if element.issuer() != ""{
        println!("{}) {} - {}: {}",i+1,element.issuer(),element.label(),get_good_otp_code(&element));
    }else{
        println!("{}) {}: {}",i+1,element.label(),get_good_otp_code(&element));
    }
}

fn get_good_otp_code(element: &database_loader::OTPElement) -> String {
    let otp = make_totp(
        &element.secret(), //we have replaced '=' in this method
               element.period(), 0).unwrap();
    let mut s_otp = otp.to_string();

    while s_otp.len() < element.digits() as usize {
        s_otp = String::from("0") + &s_otp;
    }
    s_otp
}