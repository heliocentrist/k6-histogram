extern crate csv;
extern crate serde_json;
extern crate chrono;

//use std::io;
//use std::process;
use std::fs::File;
use std::io::{BufReader, BufRead};

use serde_json::{Value, Error};

use chrono::prelude::*;

#[derive(Debug)]
#[derive(PartialEq)]
struct Point {
    time: DateTime<Utc>,
    duration: f64,
    vu: String,
    url: String,
    metric: String
}

fn parse_entry(entry: & str) -> Result<Point, Error> {
    let v: Value = serde_json::from_str(entry).unwrap();

    let time = v["data"]["time"].as_str().unwrap_or("2018-01-26T14:52:19.80265065Z").parse::<DateTime<Utc>>().unwrap();
    let duration = v["data"]["value"].as_f64().unwrap_or(99999_f64);
    let vu = v["data"]["tags"]["vu"].as_str().unwrap_or("").to_owned();
    let url = v["data"]["tags"]["name"].as_str().unwrap_or("").to_owned();
    let metric = v["metric"].as_str().unwrap_or("").to_owned();

    Ok(Point { time, duration, vu, url, metric })
}

#[test]
fn test_parse_entry() {

    let parsed = parse_entry(r#"{"type":"Point","data":{"time":"2018-01-26T14:52:19.80265065Z","value":314.131923,"tags":{"group":"","iter":"0","method":"POST","name":"http://de-tps-receptor-dev-de-dev.bright-shopper.nl:80/v1/loyalties/platform/programs/program-0/collects/passive","proto":"HTTP/1.1","status":"200","url":"http://de-tps-receptor-dev-de-dev.bright-shopper.nl:80/v1/loyalties/platform/programs/program-0/collects/passive","vu":"16"}},"metric":"http_req_duration"}"#);

    let time = "2018-01-26T14:52:19.80265065Z".parse::<DateTime<Utc>>().unwrap();
    let vu = "16".to_owned();
    let url = "http://de-tps-receptor-dev-de-dev.bright-shopper.nl:80/v1/loyalties/platform/programs/program-0/collects/passive".to_owned();
    let metric = "http_req_duration".to_owned();
    let duration = 314.131923;

    let expected: Result<Point, Error> = Ok(Point { time, duration, vu, url, metric});

    assert_eq!(parsed.unwrap(), expected.unwrap());
}

fn put_in_histo(histo: &mut [i32], thresholds: &[f64], value: f64) -> () {
    for i in 0..6 {
        if value < thresholds[i] {
            histo[i] = histo[i] + 1;
            break;
        }
    }
}

#[test]
fn test_put_in_histo() {
    let mut histo = [0, 0, 0, 0, 0, 0, 0];
    let thresholds = [300_f64, 400_f64, 500_f64, 600_f64, 700_f64, 800_f64, 1000_f64];

    put_in_histo(&mut histo, &thresholds, 399_f64);

    assert_eq!(histo, [0, 1, 0, 0, 0, 0, 0])
}

#[test]
fn test_print_histo() {
    let histo = [0, 100, 0, 200, 0, 300, 0];
    let thresholds = [300_f64, 400_f64, 500_f64, 600_f64, 700_f64, 800_f64, 1000_f64];

    let v = print_histo(&histo, &thresholds);

    assert_eq!(v, vec!["300: ", "400: ******", "500: ", "600: *************", "700: ", "800: ********************", "1000: "])
}

fn process_file(path: & str) -> Result<([i32;7], [f64;7]), std::io::Error> {
    let input = File::open(path)?;
    let buffered = BufReader::new(input);

    let mut histo = [0; 7];
    let thresholds = [300_f64, 400_f64, 500_f64, 600_f64, 700_f64, 800_f64, 1000_f64];

    for line in buffered.lines() {
        let l = line?;

        if !l.ends_with(r#""metric":"http_req_duration"}"#) {
            continue;
        }

        let p = parse_entry(&l)?;

        if p.metric == "http_req_duration" {
            put_in_histo(&mut histo, &thresholds, p.duration)
        }
    }

    // println!("{:?}", histo);

    Ok((histo, thresholds))
}

fn print_histo(histo: &[i32], thresholds: &[f64]) -> Vec<String> {

    let max_histo = histo.iter().max().unwrap();

    let max_width = 20;

    let mut vec = Vec::with_capacity(thresholds.len());

    for i in 0..(thresholds.len()) {

        let histo_value = histo[i];

        let bar_length = (max_width as f64 * (histo_value as f64 / *max_histo as f64)) as i32;

        let mut s = String::with_capacity(max_width as usize);

        for _ in 0..bar_length {
            s.push('*');
        }

        vec.push(format!("{}: {}", thresholds[i], s));
    }

    vec
}

fn main() {

    let filename = std::env::args().nth(1);

    if filename.is_none() {
        println!("Please provide file name");
        return;
    }

    let histo_res = process_file(filename.unwrap().as_str()).unwrap();

    let histo = histo_res.0;
    let thresholds = histo_res.1;

    let vec = print_histo(&histo, &thresholds);

    for s in vec {
        println!("{}", s);
    }
}
