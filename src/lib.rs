use std::fs::File;
use std::path::{PathBuf, Path};
use reqwest;
use std::io::prelude::*;
use error_chain::error_chain;
use std::io::copy;
use std::process::{ExitCode, Termination};
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use polars::df;
use polars::error::PolarsResult;
use polars::frame::DataFrame;
use polars::prelude::NamedFrom;
use tempfile::Builder;

pub const ISS_OEM_URL: &str = "https://nasa-public-data.s3.amazonaws.com/iss-coords/current/ISS_OEM/ISS.OEM_J2K_EPH.txt";

#[derive(Debug,Default)]
pub struct Satellite {
    name: String,
    id: String,
    pub trajectory_summary: String,
    pub meta_summary: String,
    pub coordinates: DataFrame,
    pub x_coord_vec: Vec<f64>,
    pub y_coord_vec: Vec<f64>,
    pub z_coord_vec: Vec<f64>,
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
#[tokio::main]
pub async fn download_file(url: &str) -> Result<String> {
    let tmp_dir = Builder::new().prefix("example").tempdir()?;

        //let path_to_crate= env!("CARGO_MANIFEST_DIR");
        //let download_path = PathBuf::from(path_to_crate).join(Path::new("./src/download_data/"));

    let target = url;
    let response = reqwest::get(target).await?;

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        let fname = tmp_dir.path().join(fname);
        //let fname = download_path.as_path().join(fname);
        //println!("will be located under: '{:?}'", fname);
        File::create(fname)?
    };

    let content =  response.text().await?;
    copy(&mut content.as_bytes(), &mut dest)?;


    Ok(content)

}

pub fn construct_oem(content: &String) -> Satellite {

    let mut sat = Satellite::new();

    let mut previous_token = "nothing";

    let mut meta_body_vec: Vec<String>= Vec::new();
    let mut traj_body_vec: Vec<String>= Vec::new();
    let mut count = 1;

    let mut count_vec: Vec<i32> = Vec::new();
    let mut date_time_vec: Vec<DateTime<FixedOffset>> = Vec::new();
    let mut x_coord_vec: Vec<f64> = Vec::new();
    let mut y_coord_vec: Vec<f64> = Vec::new();
    let mut z_coord_vec: Vec<f64> = Vec::new();

    let gmt_offset = FixedOffset::west_opt(0).unwrap();

    for line in content.lines().take(60) {
        let tokens: Vec<&str> = line.split_whitespace().collect();

        let search_meta_start = "META_START";
        let search_meta_end = "META_END";
        let search_comment = "COMMENT";
        let search_comment_source = "Source";
        let search_comment_trajectory = "TRAJECTORY";
        let search_comment_end = "End";
        let coordinates_start = "Coordinates Start";

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


        match previous_token {
            "TRAJECTORY" => {
                //let joined_line: Vec<&str> = line.split_whitespace().collect();
                let mut joined_str: String = String::from(line);
                let joined_str= joined_str.replace("COMMENT", "");
                traj_body_vec.push(joined_str);
            },
            "META_START" => {
                let mut joined_str: String = String::from(line);
                let joined_str= joined_str.replace("COMMENT", "");
                meta_body_vec.push(joined_str);
            },
            "META_END" => {
                let mut joined_str: String = String::from(line);
                let joined_str= joined_str.replace("COMMENT", "");
                meta_body_vec.push(joined_str);
            },
            "End" => {
                // If previous comment is End, then the future vector information begins.
                previous_token = coordinates_start;
            },
            "Coordinates Start" => {

                let time_stamp = tokens.get(0).unwrap().clone();

                let datetime = NaiveDateTime::parse_from_str(time_stamp, "%Y-%m-%dT%H:%M:%S.%3f").unwrap();
                let gmt_datetime = gmt_offset.from_local_datetime(&datetime).unwrap();

                let x_coord_str = tokens.get(1).unwrap();
                let x_coord = x_coord_str.parse::<f64>().unwrap_or_else(|_| panic!("Failed to parse string as f64"));

                let y_coord_str = tokens.get(2).unwrap();
                let y_coord = y_coord_str.parse::<f64>().unwrap_or_else(|_| panic!("Failed to parse string as f64"));

                let z_coord_str = tokens.get(3).unwrap();
                let z_coord = z_coord_str.parse::<f64>().unwrap_or_else(|_| panic!("Failed to parse string as f64"));

                count_vec.push(count);
                date_time_vec.push(gmt_datetime);
                x_coord_vec.push(x_coord);
                y_coord_vec.push(y_coord);
                z_coord_vec.push(z_coord);

                count += 1;


            },

            _ => { } ,
        }


    }

    let coord_df: PolarsResult<DataFrame> = df!(
        "counts"=> &count_vec,
        "x coordinates"=> &x_coord_vec,
        "y coordinates"=> &y_coord_vec,
        "z coordinates"=> &z_coord_vec,
    );

    sat.coordinates = coord_df.unwrap();

    // Cannot seem to get coordinates as Vec<64> out of dataframe downstream
    // Temporary workaround
    sat.x_coord_vec = x_coord_vec;
    sat.y_coord_vec = y_coord_vec;
    sat.z_coord_vec = z_coord_vec;


    sat.meta_summary = String::new();
    for line in meta_body_vec.iter() {
        sat.meta_summary.push_str(line);
        sat.meta_summary.push_str("\n");
    }

    sat.trajectory_summary = String::new();
    for line in traj_body_vec.iter() {
        sat.trajectory_summary.push_str(line);
        sat.trajectory_summary.push_str("\n");
    }


    // println!("{}", sat.meta_summary);
    // println!("{}", sat.trajectory_summary);
    // println!("\n{}", sat.coordinates);


    sat

}
