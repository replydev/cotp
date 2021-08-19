use totp_lite::{totp_custom, Sha1, Sha256, Sha512};
use data_encoding::BASE32_NOPAD;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn make_totp(secret: &str, algorithm: &str, digits: u64) -> String {
    let seconds: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    totp_gen(secret,algorithm,30,digits as u32,seconds)
}

fn totp_gen(secret: &str, algorithm: &str, time_step: u64, digits: u32, time: u64) -> String{
    let secret_bytes = BASE32_NOPAD.decode(secret.as_bytes()).expect("Failed to decode BASE32 Secret");
    return match algorithm {
        "SHA256" => totp_custom::<Sha256>(time_step, digits, &secret_bytes, time),
        "SHA512" => totp_custom::<Sha512>(time_step, digits, &secret_bytes, time),
        _ => totp_custom::<Sha1>(time_step,digits,&secret_bytes,time),
    }
}

#[cfg(test)]
mod tests{
    use crate::otp::otp_maker::{totp_gen};

    #[test]
    fn test_totp(){
        assert_eq!(totp_gen("BASE32SECRET3232","SHA1",30,6,0),"260182");
    }
}
