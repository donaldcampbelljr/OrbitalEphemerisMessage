use error_chain::error_chain;
use reqwest;
use OrbitalEphemerisMessage::{construct_oem, download_file, ISS_OEM_URL,Satellite, Error};

fn main() -> Result<Satellite, OrbitalEphemerisMessage::Error> {

    let url = ISS_OEM_URL;

    let content: Result<String, OrbitalEphemerisMessage::Error> = download_file(url);

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
