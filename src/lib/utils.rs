use crate::error::Error;

pub fn check_zip_code(arg: &str) -> Result<i32, Error> {
    if arg.len() == 5 {
        match arg.parse::<i32>() {
            Ok(zip_code) => Ok(zip_code),
            Err(_) => Err(Error::Invalid("The zip code provided is invalid".into())),
        }
    } else {
        Err(Error::Invalid("The zip code provided is not five digits".into()))
    }
}
