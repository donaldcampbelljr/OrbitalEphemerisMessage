use error_chain::error_chain;
use reqwest;


use rs_OrbitalEphemerisMessages::{construct_oem, download_file, Satellite, Error};

const ISS_OEM_URL: &str = "https://nasa-public-data.s3.amazonaws.com/iss-coords/current/ISS_OEM/ISS.OEM_J2K_EPH.txt";


fn main() -> Result<Satellite, rs_OrbitalEphemerisMessages::Error> {

    let url = ISS_OEM_URL;

    let content: Result<String, rs_OrbitalEphemerisMessages::Error> = download_file(url);

    let sat = match content {
        Ok(content) => construct_oem(&content),
        Err(error) => {
            println!("Error downloading content: {}", error);
            // Return a default Satellite value if there was an error
            Satellite::default()
        }
    };

    Ok(sat)

}
