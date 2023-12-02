use std::fs::File;
use std::io::Read;
use std::io;
use reqwest;
use std::io::prelude::*;
use error_chain::error_chain;
use std::io::copy;
use std::process::{ExitCode, Termination};
use tempfile::Builder;

#[derive(Debug,Default)]
pub struct Satellite {
    name: String,
    id: String,
    trajectory_summary: String,
    meta_summary: String,
}

impl Satellite {
    /// Constructs a new instance of [`Satellite`].
    pub fn new() -> Self {
        Self::default()
    }

}

impl Termination for Satellite {
    fn report(self) -> ExitCode {
        ExitCode::SUCCESS
    }
}

error_chain! {
     foreign_links {
         Io(std::io::Error);
         HttpRequest(reqwest::Error);
     }
}

const ISS_OEM_URL: &str = "https://nasa-public-data.s3.amazonaws.com/iss-coords/current/ISS_OEM/ISS.OEM_J2K_EPH.txt";

#[tokio::main]
async fn main() -> Result<Satellite> {
    let tmp_dir = Builder::new().prefix("example").tempdir()?;
    let target = ISS_OEM_URL;
    let response = reqwest::get(target).await?;

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        println!("file to download: '{}'", fname);
        let fname = tmp_dir.path().join(fname);
        println!("will be located under: '{:?}'", fname);
        File::create(fname)?
    };

    let content =  response.text().await?;

    copy(&mut content.as_bytes(), &mut dest)?;

    //println!("{}", content);
    let sat = construct_oem(&content);

    Ok((sat))

}

fn construct_oem(content: &String) -> Satellite {

    let mut sat = Satellite::new();

    // META_START
    // META_END
    // COMMENT Source
    //  TRAJECTORY
    // COMMENT End sequence of events
    // 2022-02-18T12:00:00.000 6432.338357027310 1810.414013580070 1210.742166479110 -0.28169387337306 4.94773870038605 -5.85002385833392
    // 2022-02-18T12:04:00.000 6130.995481102160 2917.381297198860 -220.384941250909 -2.21387475002967 4.22061672892853 -6.00280929188885

    let mut previous_token = "nothing";

    let mut meta_body_vec: Vec<&str>= Vec::new();

    for line in content.lines().take(50) {
        let tokens: Vec<&str> = line.split_whitespace().collect();

        let search_meta_start = "META_START";
        let search_meta_end = "META_END";
        let search_comment = "COMMENT";
        let search_comment_source = "Source";
        let search_comment_trajectory = "TRAJECTORY";
        let search_comment_end = "End";

        if tokens.get(0).unwrap_or(&"").contains(search_meta_start) {
            //println!("Line contains '{}' at the beginning: {}", search_meta_start, line);
            previous_token = search_meta_start;
        }

        if tokens.get(0).unwrap_or(&"").contains(search_meta_end) {
            //println!("Line contains '{}' at the beginning: {}", search_meta_end, line);
            previous_token = search_meta_end;
        }

        if tokens.get(0).unwrap_or(&"").contains(search_comment) {
            //println!("Line contains '{}' at the beginning: {}", search_comment, line);

            if tokens.len() > 1 && tokens.get(1).unwrap_or(&"").contains(search_comment_source) {
                //println!("Line contains '{}' at the second position: {}", search_comment_source, line);
                previous_token = search_comment_source
            }

            if tokens.len() > 1 && tokens.get(1).unwrap_or(&"").contains(search_comment_trajectory) {
                //println!("Line contains '{}' at the second position: {}", search_comment_trajectory, line);
                previous_token = search_comment_trajectory;
            }

            if tokens.len() > 1 && tokens.get(1).unwrap_or(&"").contains(search_comment_end) {
                //println!("Line contains '{}' at the second position: {}", search_comment_end, line);
                previous_token = search_comment_end;
            }

        }

        let breaking = "2022-02-18T12:00:00.000";

        if tokens.get(0).unwrap_or(&"").contains(breaking) {
            //println!("Line contains '{}' at the beginning: {}", breaking, line);
            previous_token = breaking;
        }

        //println!("------>^^^^^^^^^^{}",previous_token);

        match previous_token {
            "TRAJECTORY" => {
                println!("Processing trajectory data");
            },
            "META_START" => {
                meta_body_vec.push(line);
            },
            "META_END" => {
                meta_body_vec.push(line);
            },
            "End" => {
                println!("Comments End");
            },
            "2022-02-18T12:00:00.000" => {
                println!("Found beginning of");
            },
            _ => { } ,
        }


    }


    sat.meta_summary = String::new();
    for line in meta_body_vec.iter() {
        sat.meta_summary.push_str(line);
        sat.meta_summary.push_str("\n");
    }

    // sat.meta_summary = meta_body;

    println!("{}", sat.meta_summary);

    sat

}