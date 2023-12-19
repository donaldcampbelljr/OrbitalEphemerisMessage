use std::fs;
use std::path::{Path, PathBuf};
use std::fs::{Metadata, OpenOptions};
use std::fs::*;
use std::io::{BufRead, BufReader};

use error_chain::error_chain;
use reqwest;
use OrbitalEphemerisMessage::{construct_oem, download_file, ISS_OEM_URL,Satellite, Error};

use chrono::{DateTime, FixedOffset, Utc};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() -> Result<Satellite, OrbitalEphemerisMessage::Error> {

    let url = ISS_OEM_URL;
    println!("Downloading ISS OEM Data from: {}", ISS_OEM_URL);
    //let result =check_file();

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

// fn check_file() -> Option<bool> {
//     // Get the file path based on the cargo manifest directory.
//     let path_to_crate = env!("CARGO_MANIFEST_DIR");
//     let file_path = PathBuf::from(path_to_crate).join(Path::new("./src/download_data/ISS.OEM_J2K_EPH.txt"));
//
//     // let now = SystemTime::now();
//     // let seconds = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
//
//     // Try to get the file's metadata.
//     match fs::metadata(&file_path) {
//         Ok(metadata) => {
//             // Check if the file is older than 1 day.
//             // Open the file
//             let file = File::open(file_path).expect("Error opening file");
//
// // Create a buffered reader
//             let reader = BufReader::new(file);
//
// // Iterate lines of the file
//             let lines = reader.lines();
//
//             let creation_date: DateTime<FixedOffset>= Default::default();
//
//             // Find the line starting with "CREATION_DATE"
//             for line in lines {
//                 let line = line.expect("Failed to read line");
//                 let tokens: Vec<&str> = line.split_whitespace().collect();
//                 if tokens.get(0).unwrap_or(&"").contains("CREATION_DATE") {
//                 // Check if the line starts with "CREATION_DATE"
//                     // Extract the date string after the equal sign
//                     let date_str = tokens.get(1).unwrap();
//
//                     // Parse the date string to a date object
//                     let creation_date = DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S.%f") //2023-11-29T16:26:35.171
//                         .expect("Error parsing creation date");
//
//                     // Print or use the creation date object
//                     println!("Creation date: {}", creation_date);
//                     // ... process the date further ...
//
//                     break;
//                 }
//             }
//
//             let one_day_ago = Utc::now();
//
//             if creation_date < one_day_ago {
//                 Some(true) // File exists and is older than 1 day
//             } else {
//                 Some(false) // File exists but is not older than 1 day
//             }
//         }
//         Err(_) => None, // File does not exist
//     }
// }
