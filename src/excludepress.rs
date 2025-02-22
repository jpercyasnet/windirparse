use rfd::FileDialog;
use std::path::{Path, PathBuf};
use std::process::Command as stdCommand;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn excludepress (excludeval: String) -> (u32, String, String) {
     let mut errcode: u32 = 0;
     let mut errstring: String  = "got file".to_string();
     let mut new_input: String;
     if Path::new(&excludeval).exists() {
         let getpath = PathBuf::from(&excludeval);
         let getdir = getpath.parent().unwrap();
         new_input = getdir.to_str().unwrap().to_string();
     } else {
         new_input = "/".to_string();
     }
     let newfile = FileDialog::new()
         .set_directory(&new_input)
         .pick_file();
     if newfile == None {
         errstring = "error getting exclude file -- possible cancel key hit".to_string();
         errcode = 1;
     } else {
         new_input = newfile.as_ref().expect("REASON").display().to_string();
         if !Path::new(&new_input).exists() {
             errstring = format!("The exclude file does not exist: {}", new_input);
             errcode = 2;
         } else {
             let outputy = stdCommand::new("wc")
                   .arg("-l")
                   .arg(&new_input)
                   .output()
                   .expect("failed to execute process");
             let strouty = String::from_utf8_lossy(&outputy.stdout);
             let vecouty: Vec<&str> = strouty.split(" ").collect();
             let numlinesy: i64 = vecouty[0].parse().unwrap_or(-9999);
             if numlinesy == -9999 {
                 errstring = format!("size of {} is invalid for wc -l command call for exclude file", vecouty[0]);
                 errcode = 3;
             } else {
                 let exrows_num = numlinesy as u64;
                 if exrows_num < 2 {
                     errstring = format!("size of {} is less than 2 for {}", exrows_num, new_input);
                     errcode = 4;
                 } else {
                     let filey = File::open(new_input.clone()).unwrap();
                     let mut readery = BufReader::new(filey);
                     let mut lineex = String::new();
                     let mut linenumy: u64 = 0;
                     loop {
                         match readery.read_line(&mut lineex) {
                            Ok(bytes_read) => {
                                if bytes_read == 0 {
                                    errstring = "exclude file is has no records".to_string();
                                    errcode = 5;
                                    break;
                                }
                                linenumy = linenumy + 1;
                                if linenumy == 1 {
                                    if lineex.trim().to_string() != "exclude file".to_string() {
                                        errstring = format!("first line of exclude file is not valid: {}", lineex);
                                        errcode = 6;
                                    }
                                } else {
                                    break;
                                }
                            }
                            Err(err) => {  
                                errstring = format!("error of {} reading exclude file {}", err, new_input);
                                errcode = 7;
                                break;
                            }
                         };
                     }
                 }
             }
         }
     } 
    (errcode, errstring, new_input)
}

