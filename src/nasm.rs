use std::{
    process::Command,
    io::Write,
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


pub struct Nasm;
impl Nasm {
    pub fn errors<S:Into<String>>(s:S)->Result<Vec<NasmError>,String> {
        let mut command=Command::new("nasm");
        let string=s.into();
        let format=if string.contains("lsp: none") {
            "bin"
        } else {
            "elf64"
        };
        let mut named_tmp=NamedTempFile::new().unwrap();
        named_tmp.write(&string.bytes().collect::<Vec<u8>>()).unwrap();
        let name=named_tmp.path().to_str().unwrap();
        command.args(&["-o","/dev/null","-f",format,name]);
        if let Ok(output)=command.output() {
            let stderr=String::from_utf8(output.stderr).unwrap();
            let stderr=stderr.trim().to_string();
            let mut errors=Vec::new();
            if stderr.contains(name) {
                for line in stderr.split('\n') {
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
