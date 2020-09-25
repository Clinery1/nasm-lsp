use std::{
    process::Command,
    path::PathBuf,
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


pub struct Nasm {
    tempdir:PathBuf,
}
impl Nasm {
    pub fn new()->Nasm {
        let pwd=std::env::current_dir().unwrap();
        let tempdir_name=pwd.to_str().unwrap().chars().map(|input|{if input=='/'{'-'}else{input}}).collect::<String>();
        let tempdir=PathBuf::from(format!("/tmp/NASMLSP{}",tempdir_name));
        if !tempdir.exists() {
            std::fs::create_dir(tempdir.clone()).expect("Could not create tempdir");
        }
        let mut c=Command::new("cp");
        c.arg("-r");
        for file in std::fs::read_dir(pwd.clone()).unwrap() {
            let file=file.unwrap();
            c.arg(&format!("{}",file.path().to_str().unwrap()));
        }
        c.arg(&format!("/tmp/NASMLSP{}/",tempdir_name));
        c.status().unwrap();
        Nasm {
            tempdir,
        }
    }
    pub fn update_files<T:Into<String>>(&self,filename:T,text:String) {
        let pwd=std::env::var("PWD").unwrap();
        let pathname=filename.into();
        eprintln!("pwd: {}",pwd);
        let mut filename=pathname.clone().split_off(pwd.len());
        if filename.starts_with("/src/") {
            filename=filename.split_off(5);
        }
        let path=format!("{}/{}",self.tempdir.to_str().unwrap(),filename);
        eprintln!("{}",path);
        std::fs::write(path,text).unwrap();
    }
    pub fn get_errors<T:Into<String>>(&self,filename:T)->Result<Vec<NasmError>,String> {
        let mut command=Command::new("nasm");
        let pwd=std::env::var("PWD").unwrap();
        let pathname=filename.into();
        let mut filename=pathname.clone().split_off(pwd.len());
        if filename.starts_with("/src/") {
            filename=filename.split_off(5);
        }
        let new_pathname=format!("{}/{}",self.tempdir.to_str().unwrap(),filename);
        command.env("PWD",pwd.clone());
        command.args(&["-o","/dev/null","-f elf64",&new_pathname]);
        eprintln!("filename: {},PWD: {},new pathname: {}",pathname,pwd,new_pathname);
        if let Ok(output)=command.output() {
            let stderr=String::from_utf8(output.stderr).unwrap();
            let stderr=stderr.trim().to_string();
            let mut errors=Vec::new();
            for line in stderr.split('\n') {
                if line.len()>0 {
                    eprintln!("Line: {}",line);
                    errors.push(NasmError::from_string(line.to_string()));
                }
            }
            return Ok(errors);
        } else {
            return Err("Could not run NASM".to_string());
        }
    }
}


#[derive(Clone,Debug,Default,Eq,PartialEq)]
pub struct NasmError {
    pub line:usize,
    pub error_type:ErrorType,
    pub contents:String,
}
impl NasmError {
    pub fn from_string(input:String)->NasmError {
        let mut error:NasmError=NasmError::default();
        let mut chars=input.chars().collect::<Vec<char>>();
        for (i,char) in chars.iter().enumerate() {
            if *char==':'&&i>0 {
                chars=chars.split_off(i+1);
                break;
            }
        }
        let mut line=String::new();
        for (i,char) in chars.iter().enumerate() {
            if *char==':'&&i>0 {
                chars=chars.split_off(i+1);
                break;
            }
            line.push(*char);
        }
        error.line=line.trim().parse::<usize>().unwrap();
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
