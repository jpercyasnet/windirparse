use std::path::Path;

pub fn execpress (windir_value: String, rows_num: u64, exclude_value: String) -> (u32, String) {
     let mut errcode: u32 = 0;
     let mut errstring: String = "all good and now process execution".to_string();
     if !Path::new(&exclude_value).exists() {
         errstring = "the exclude file does not exist".to_string();
         errcode = 1;
     } else {
         if Path::new(&windir_value).exists() {
             if rows_num < 11 {
                 errcode = 2;
                 errstring = "The number of rows is less than 11".to_string();
             }
         } else {
             errstring = "the window dir file does not exist".to_string();
             errcode = 3;
         }
     }  
     (errcode, errstring)
}

