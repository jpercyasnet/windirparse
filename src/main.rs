use iced::widget::{button, column, row, text, progress_bar, Space};
use iced::{Alignment, Element, Task, Color};
use iced::theme::{Theme};
use iced_futures::futures;
use futures::channel::mpsc;
extern crate chrono;
use std::path::Path;
use std::io::{Write, BufRead, BufReader};
use std::fs::File;
use std::time::Duration as timeDuration;
use std::thread::sleep;
use chrono::{Duration, Utc};
use chrono::prelude::*;
use dateparser::parse_with_timezone;
//use dateparser::DateTimeUtc;
//use dateparser::parse;
use chrono::offset::Utc as Utcx;
mod get_winsize;
mod inputpress;
mod execpress;
mod excludepress;
use get_winsize::get_winsize;
use inputpress::inputpress;
use execpress::execpress;
use excludepress::excludepress;

pub fn main() -> iced::Result {

     let mut widthxx: f32 = 1350.0;
     let mut heightxx: f32 = 750.0;
     let (errcode, errstring, widtho, heighto) = get_winsize();
     if errcode == 0 {
         widthxx = widtho as f32 - 20.0;
         heightxx = heighto as f32 - 75.0;
         println!("{}", errstring);
     } else {
         println!("**ERROR {} get_winsize: {}", errcode, errstring);
     }
     iced::application(Windirparse::title, Windirparse::update, Windirparse::view)
        .window_size((widthxx, heightxx))
        .theme(Windirparse::theme)
        .run_with(Windirparse::new)

}

struct Windirparse {
    windir_value: String,
    excludefile_value: String,
    mess_color: Color,
    msg_value: String,
    rows_num: u64,
    do_progress: bool,
    progval: f32,
    tx_send: mpsc::UnboundedSender<String>,
    rx_receive: mpsc::UnboundedReceiver<String>,
}

#[derive(Debug, Clone)]
enum Message {
    WindirPressed,
    ExcludeFilePressed,
    ExecPressed,
    ExecxFound(Result<Execx, Error>),
    ProgressPressed,
    ProgRtn(Result<Progstart, Error>),
}

impl Windirparse {
    fn new() -> (Windirparse, iced::Task<Message>) {
        let (tx_send, rx_receive) = mpsc::unbounded();
        ( Self { windir_value: "--".to_string(), excludefile_value: "--".to_string(), msg_value: "no message".to_string(),
               rows_num: 0, mess_color: Color::from([0.0, 0.0, 1.0]), 
               do_progress: false, progval: 0.0, tx_send, rx_receive,
 
          },
          Task::none()
        )
    }

    fn title(&self) -> String {
        String::from("Windir parse of file list for just name and size, but creates another with time -- iced")
    }

    fn update(&mut self, message: Message) -> Task<Message>  {
        match message {
            Message::WindirPressed => {
               let inputstr: String = self.windir_value.clone();
               let (errcode, errstr, newinput) = inputpress(inputstr);
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   if Path::new(&newinput).exists() {
                       self.mess_color = Color::from([0.0, 1.0, 0.0]);
                       self.windir_value = newinput.to_string();
                       self.rows_num = 0;
                       let mut bolok = true;
                       let file = File::open(newinput).unwrap();
                       let mut reader = BufReader::new(file);
                       let mut line = String::new();
                       let mut linenum: u64 = 0;
                       let mut linesav = " ".to_string();
                       loop {
                          match reader.read_line(&mut line) {
                             Ok(bytes_read) => {
                                 // EOF: save last file address to restart from this address for next run
                                 if bytes_read == 0 {
                                     break;
                                 }
                                 linesav = line.clone();
                                 linenum = linenum + 1;
                                 line.clear();
                             }
                             Err(err) => {
                                 self.msg_value = format!("error {:?} reading window dir file line {} {}", err, linenum, linesav);
                                 self.mess_color = Color::from([1.0, 0.0, 0.0]); 
                                 bolok = false;   
                                 break;
                             }
                          };
                       }
                       if bolok {
                           self.rows_num = linenum;
                           self.mess_color = Color::from([0.0, 1.0, 0.0]);
                           self.msg_value = "got window dir file and retrieved its number of rows".to_string();
                       } 
                   } else {
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                       self.msg_value = format!("window dir file does not exist: {}", newinput);
                   }
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               Task::none()
            }
            Message::ExcludeFilePressed => {
               let inputstr: String = self.excludefile_value.clone();
               let (errcode, errstr, newinput) = excludepress(inputstr);
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
                   self.excludefile_value = newinput.to_string();
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               Task::none()
            }
            Message::ExecPressed => {
               let (errcode, errstr) = execpress(self.windir_value.clone(), self.rows_num.clone(), self.excludefile_value.clone());
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
                   Task::perform(Execx::execit(self.windir_value.clone(), self.rows_num.clone(), self.excludefile_value.clone(), self.tx_send.clone()), Message::ExecxFound)
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
                   Task::none()
               }
            }
            Message::ExecxFound(Ok(exx)) => {
               self.msg_value = exx.errval.clone();
               if exx.errcd == 0 {
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               self.do_progress = false;
               self.progval = 0.0;
               Task::none()
            }
            Message::ExecxFound(Err(_error)) => {
               self.msg_value = "error in copyx copyit routine".to_string();
               self.mess_color = Color::from([1.0, 0.0, 0.0]);
               Task::none()
            }
            Message::ProgressPressed => {
                   self.do_progress = true;
                   Task::perform(Progstart::pstart(), Message::ProgRtn)
            }
            Message::ProgRtn(Ok(_prx)) => {
              if self.do_progress {
                let mut inputval  = " ".to_string();
                let mut bgotmesg = false;
                while let Ok(Some(input)) = self.rx_receive.try_next() {
                   inputval = input;
                   bgotmesg = true;
                }
                if bgotmesg {
                    let progvec: Vec<&str> = inputval[0..].split("|").collect();
                    let lenpg1 = progvec.len();
                    if lenpg1 == 3 {
                        let prog1 = progvec[0].to_string();
                        if prog1 == "Progress" {
                            let num_int: i32 = progvec[1].parse().unwrap_or(-9999);
                            if num_int == -9999 {
                                println!("progress numeric not numeric: {}", inputval);
                            } else {
                                let dem_int: i32 = progvec[2].parse().unwrap_or(-9999);
                                if dem_int == -9999 {
                                    println!("progress numeric not numeric: {}", inputval);
                                } else {
                                    self.progval = 100.0 * (num_int as f32 / dem_int as f32);
                                    self.msg_value = format!("Convert progress: {} of {}", num_int, dem_int);
                                    self.mess_color = Color::from([0.0, 0.0, 1.0]);
                                }
                            }
                        } else {
                            println!("message not progress: {}", inputval);
                        }
                    } else {
                        println!("message not progress: {}", inputval);
                    }
                }             
                Task::perform(Progstart::pstart(), Message::ProgRtn)
              } else {
                Task::none()
              }
            }
            Message::ProgRtn(Err(_error)) => {
                self.msg_value = "error in Progstart::pstart routine".to_string();
                self.mess_color = Color::from([1.0, 0.0, 0.0]);
               Task::none()
            }

        }
    }

    fn view(&self) -> Element<Message> {
        column![
            row![text("Message:").size(20),
                 text(&self.msg_value).size(30).color(*&self.mess_color),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![button("window dir /s input file Button").on_press(Message::WindirPressed),
                 text(&self.windir_value).size(20).width(1000)
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![button("exclude input file Button").on_press(Message::ExcludeFilePressed),
                 text(&self.excludefile_value).size(20).width(1000)
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![text(format!("number of rows: {}", self.rows_num)).size(20), Space::with_width(100),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![Space::with_width(200),
                 button("Exec Button").on_press(Message::ExecPressed),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![button("Start Progress Button").on_press(Message::ProgressPressed),
                 progress_bar(0.0..=100.0,self.progval),
                 text(format!("{:.1}%", &self.progval)).size(30),
            ].align_y(Alignment::Center).spacing(5).padding(10),
         ]
        .padding(5)
        .align_x(Alignment::Start)
        .into()
    }

    fn theme(&self) -> Theme {
       Theme::Dracula
    }
}

#[derive(Debug, Clone)]
struct Execx {
    errcd: u32,
    errval: String,
}

impl Execx {
    async fn execit(windir_value: String, rows_num: u64, excludefile_value: String, tx_send: mpsc::UnboundedSender<String>,) -> Result<Execx, Error> {
     let mut errstring  = "begin of async".to_string();
     let mut errcode: u32 = 0;
     let mut bolok = true;
     let numrows: u64 = rows_num;
     let fileex = File::open(excludefile_value.clone()).unwrap();
     let mut readerex = BufReader::new(fileex);
     let mut lineex = String::new();
     let mut lineexnum: u64 = 0;
     let mut vecexcludef: Vec<String> = Vec::new();
     let mut vecexcluded: Vec<String> = Vec::new();
     let mut outseq: u32 = 1;
     let mut linegood: u64 = 0;
     let mut lineexclude: u64 = 0;
     loop {
           match readerex.read_line(&mut lineex) {
               Ok(bytes_read) => {
                   if bytes_read == 0 {
                       break;
                   }
                   lineexnum = lineexnum + 1;
                   if lineexnum > 1 {
                       let excl: String = lineex.trim().to_string();
                       if excl.len() < 3 {
                           errstring  = format!("exclusion less than 3 characters: {}", excl);
                           errcode = 1;
                           bolok = false;
                           break;
                       } else {
                           let exclv: String = excl[2..].to_string();
                           if excl[..2].to_string() == "d ".to_string() {
                               vecexcluded.push(exclv);
                           } else if excl[..2].to_string() == "f ".to_string() {
                               vecexcludef.push(exclv);
                           } else {
                               errstring  = format!("exclusion invalid first two characters {}", excl);
                               errcode = 2;
                               bolok = false;
                               break;
                           }
                       }   
                   }
                   lineex.clear();
               }
               Err(err) => {
                   errstring  = format!("error {} when reading exclusion file", err);
                   errcode = 3;
                   bolok = false;   
                   break;
               }
           };
     }
     if bolok {
         if lineexnum < 2 {
             errstring  = format!("exclusion file {} has no records", excludefile_value);
             errcode = 4;
             bolok = false;
         }
     }
     if bolok {
         let mut targetcsv: String = format!("{}_out{:02}.csv", windir_value, outseq);
         let mut excludout: String = format!("{}_excluded{:02}.excout", windir_value, outseq);
         loop {
               if !Path::new(&targetcsv).exists() && !Path::new(&excludout).exists() {
                   break;
               } else {
                   outseq = outseq + 1;
                   targetcsv = format!("{}_out{:02}.csv", windir_value, outseq);
                   excludout = format!("{}_excluded{:02}.csv", windir_value, outseq);
               }
         }          
         let mut targetcsvfile = File::create(targetcsv).unwrap();
         let mut excludefile = File::create(excludout).unwrap();
         let file = File::open(windir_value).unwrap(); 
         let mut reader = BufReader::new(file);
         let mut line = String::new();
         let mut linenum = 0;
         loop {
               match reader.read_line(&mut line) {
                    Ok(bytes_read) => {
                       if bytes_read == 0 {
                           break;
                       }
//                       if linenum > 40 {
//                           break;
//                       }
                       linenum = linenum + 1;
                       let lineparse: Vec<&str> = line[0..].split('"').collect();
                       let lineparlen = lineparse.len();
                       if lineparlen > 6 && linenum > 2 {
                           let mut datetimef = lineparse[3].to_string();
                           match parse_with_timezone(&datetimef, &Utcx) {
                             Ok(datev) => {
                                 datetimef = format!("{}", datev.format("%Y-%m-%d %H:%M:%S"));
                             }
                             Err(err) => {
                                 datetimef = format!("error {:?} date", err);
                             }
                           }
                           let sizef = lineparse[5].to_string();
                           let namefullf = lineparse[1].to_string();
                           let dirend: usize;
                           match namefullf.rfind("\\") {
                                Some(index) =>  dirend = index,
                                None =>         dirend = 0,
                           }
                           let namef: String;
                           let dirf: String;
                           if dirend < 1 {
                               namef = namefullf.clone();
                               dirf = "".to_string();
                           } else {
                               namef = namefullf[(dirend+1)..].to_string();
                               dirf = namefullf[0..(dirend)].to_string();
                           }
                           let linehdfmt = format!("{}|{}|{}|{}", namef, dirf, datetimef, sizef);
                           let mut bolex = false;
                           for strexclf in &vecexcludef {
                                if namef.contains(strexclf) {
                                    bolex = true;
                                    writeln!(&mut excludefile, "{}|f", linehdfmt).unwrap();
                                    lineexclude = lineexclude + 1;
                                    break;
                                }
                           }
                           if !bolex {
                               for strexcld in &vecexcluded {
                                    if dirf.contains(strexcld) {
                                        bolex = true;
                                        writeln!(&mut excludefile, "{}|d", linehdfmt).unwrap();
                                        lineexclude = lineexclude + 1;
                                        break;
                                    }
                               }
                           }
                           if !bolex {
                               writeln!(&mut targetcsvfile, "{}|good", linehdfmt).unwrap();
                               linegood = linegood + 1;
                           }
                       }
                       let msgx = format!("Progress|{}|{}", linenum, numrows);
                       tx_send.unbounded_send(msgx).unwrap();
                       if linenum > numrows {
                           break;
                       }
                       line.clear();
                    }
                    Err(err) => {
                       errstring = format!("error {:?} reading windows dir file", err);
                       errcode = 5;
                       break;
                    }
               }
         }
     }
     if errcode == 0 {
         errstring = format!("Completed: good: {} excluded: {} suffix number: {}", linegood, lineexclude, outseq)
     }
     Ok(Execx {
            errcd: errcode,
            errval: errstring,
        })
     }
}
#[derive(Debug, Clone)]
pub enum Error {
//    APIError,
//    LanguageError,
}

// loop thru by sleeping for 5 seconds
#[derive(Debug, Clone)]
pub struct Progstart {
//    errcolor: Color,
//    errval: String,
}

impl Progstart {

    pub async fn pstart() -> Result<Progstart, Error> {
//     let errstring  = " ".to_string();
//     let colorx = Color::from([0.0, 0.0, 0.0]);
     sleep(timeDuration::from_secs(5));
     Ok(Progstart {
//            errcolor: colorx,
//            errval: errstring,
        })
    }
}
