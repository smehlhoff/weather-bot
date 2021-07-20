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

pub fn check_station_code(station: &str) -> Result<(), Error> {
    if station.len() == 4 {
        if station.starts_with('K') {
            Ok(())
        } else {
            Err(Error::Invalid("U.S. station codes only (e.g., KSFO, KJFK)".into()))
        }
    } else {
        Err(Error::Invalid("The station code provided is invalid".into()))
    }
}
