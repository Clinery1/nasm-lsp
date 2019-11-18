use std::{
    process::Command,
    io::Write,
    //fs::File,
};
use tempfile::{
    NamedTempFile,
};


#[derive(Clone,Debug,Eq,PartialEq)]
pub enum ErrorType {
    Error,
    Warning,
    Other,
}
impl Default for ErrorType {
    fn default()->ErrorType {ErrorType::Error}
}


pub struct NasmFile {
    pub errors:Vec<NasmError>,
    pub file:NamedTempFile,
}
impl NasmFile {
    pub fn new()->NasmFile {
        NasmFile {
            errors:Vec::new(),
            file:NamedTempFile::new().expect("Could not create temp file"),
        }
    }
    pub fn update_contents(&mut self,contents:String) {
        let mut file=self.file.reopen().unwrap();
        write!(file,"{}",contents).unwrap();
    }
    /*pub fn parse_new<'a>(log:&mut File)->Result<NasmFile,String> {
        let mut ns_file=NasmFile::new();
        if let Err(string)=ns_file.parse(log) {
            return Err(string);
        }
        return Ok(ns_file);
    }*/
    pub fn parse(&mut self,/*log:&mut File*/)->Result<(),String> {
        //writeln!(log,"Started execution of NASMFILE").unwrap();
        self.errors=Vec::new();
        let mut command=Command::new("nasm");
        command.args(&["-o","/dev/null","-f","elf",self.file.path().to_str().unwrap()]);
        //writeln!(log,"Starting command execution").unwrap();
        if let Ok(output)=command.output() {
            //writeln!(log,"Command executed").unwrap();
            let stderr=String::from_utf8(output.stderr).unwrap();
            //writeln!(log,"STDERR converted").unwrap();
            let stderr=stderr.trim().to_string();
            for line in stderr.split('\n') {
                self.errors.push(NasmError::from_string(line.to_string()/*,log*/));
            }
            //writeln!(log,"Generated errors").unwrap();
        } else {
            return Err("Could not run NASM".to_string());
        }
        Ok(())
    }
}

#[derive(Clone,Debug,Default,Eq,PartialEq)]
pub struct NasmError {
    pub filename:String,
    pub line:usize,
    pub error_type:ErrorType,
    pub contents:String,
}
impl NasmError {
    pub fn from_string(input:String,/*log:&mut File*/)->NasmError {
        let mut error:NasmError=NasmError::default();
        let mut chars=input.chars().collect::<Vec<char>>();
        for (i,char) in chars.iter().enumerate() {
            if *char==':'&&i>0 {
                chars=chars.split_off(i+1);
                break;
            }
            error.filename.push(*char);
        }
        error.filename=error.filename.trim().to_string();
        let mut line=String::new();
        for (i,char) in chars.iter().enumerate() {
            if *char==':'&&i>0 {
                chars=chars.split_off(i+1);
                break;
            }
            line.push(*char);
        }
        //writeln!(log,"About to parse number: {:?}",line.trim()).unwrap();
        error.line=line.trim().parse::<usize>().unwrap();
        //writeln!(log,"Parsed Number").unwrap();
        let mut err_type=String::new();
        for (i,char) in chars.iter().enumerate() {
            if *char==':'&&i>0 {
                chars=chars.split_off(i+1);
                break;
            }
            err_type.push(*char);
        }
        error.error_type=match err_type.trim().to_lowercase().as_str() {
            "warning"=>ErrorType::Warning,
            "error"=>ErrorType::Error,
            _=>ErrorType::Other,
        };
        for i in chars {error.contents.push(i)}
        error.contents=error.contents.trim().to_string();
        return error;
    }
}
