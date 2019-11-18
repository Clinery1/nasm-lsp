mod nasm;
mod test;


use crossbeam_channel::{Sender, Receiver};
use lsp_types::{
    ServerCapabilities,
    InitializeParams,
    request::*,
    notification::*,
    Hover,
    HoverContents,
    MarkupContent,
    MarkupKind,
    TextDocumentSyncCapability,
    TextDocumentSyncKind,
    PublishDiagnosticsParams,
    Url,
    Range,
    Position,
    DiagnosticSeverity,
    Diagnostic,
};
use gen_lsp_server::{
    run_server,
    stdio_transport,
    handle_shutdown,
    RawMessage,
    RawResponse,
    RawNotification,
};
use std::{
    sync::Mutex,
    collections::HashMap,
    //fs::File,
    //io::Write,
};
use lazy_static::lazy_static;
use nasm::*;


lazy_static! {
    static ref FILES:Mutex<HashMap<String,NasmFile>>=Mutex::new(HashMap::new());
    //static ref LOG:Mutex<File>=Mutex::new(File::create("log.log").unwrap());
}


fn main() -> Result<(), failure::Error> {
    let capabilities:ServerCapabilities=ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::Full)),
        hover_provider: Some(true),
        completion_provider: None,
        signature_help_provider: None,
        definition_provider:None,
        type_definition_provider:None,
        implementation_provider:None,
        references_provider:None,
        document_highlight_provider:None,
        document_symbol_provider:None,
        workspace_symbol_provider:None,
        code_action_provider:None,
        code_lens_provider:None,
        document_formatting_provider:None,
        document_range_formatting_provider:None,
        document_on_type_formatting_provider:None,
        rename_provider:None,
        color_provider:None,
        folding_range_provider:None,
        execute_command_provider:None,
        workspace:None,
    };
    let (receiver, sender, io_threads) = stdio_transport();
    run_server(
        capabilities,
        receiver,
        sender,
        main_loop,
    )?;
    io_threads.join()?;
    Ok(())
}

fn main_loop(
    _: InitializeParams,
    receiver: &Receiver<RawMessage>,
    sender: &Sender<RawMessage>,
) -> Result<(), failure::Error> {
    //let mut log=LOG.lock().unwrap();
    for msg in receiver {
        match msg {
            RawMessage::Request(req)=>{
                match handle_shutdown(req.clone(), sender) {
                    None=>return Ok(()),
                    Some(r)=>{
                        if let Ok((id,params))=r.clone().cast::<HoverRequest>() {
                            let mut error:Option<String>=None;
                            let loc=params.text_document.uri.path();
                            let resp:RawResponse;
                            if let Ok(mut files)=FILES.lock() {
                                let mut contents:Option<String>=None;
                                //log.write(b"Got FILES\n").unwrap();
                                if let Some(nasm_file)=files.get_mut(loc) {
                                    //log.write(b"Got the file from FILES\n").unwrap();
                                    if let Err(err)=nasm_file.parse() {
                                        error=Some(err);
                                        //log.write(b"NASM parse error\n").unwrap();
                                    } else {
                                        //log.write(b"NASM parsed the file").unwrap();
                                    }
                                } else {
                                    //log.write(b"Creating new file for FILES\n").unwrap();
                                    let mut nasm_file=NasmFile::new();
                                    if let Err(err)=nasm_file.parse() {
                                        error=Some(err);
                                        //log.write(b"NASM parse error\n").unwrap();
                                    }
                                    files.insert(loc.to_string(),nasm_file);
                                }
                                let line=params.position.line;
                                if let Some(file)=files.get(loc) {
                                    //log.write(b"Got file from FILES after creation/update\n").unwrap();
                                    for error in file.errors.clone() {
                                        if error.line-1==line as usize {
                                            //log.write(format!("Got the right line! Line: {}\n",line).as_bytes()).unwrap();
                                            contents=Some(format!("{:?}: {}",error.error_type,error.contents));
                                            break;
                                        }
                                    }
                                } else {
                                    //log.write(b"Did not get file from FILES after creation/update\n").unwrap();
                                }
                                resp=RawResponse::ok::<HoverRequest>(
                                    id,
                                    &Some(Hover {
                                        contents:HoverContents::Markup(MarkupContent {
                                            kind:MarkupKind::PlainText,
                                            value:if let Some(err)=error {
                                                format!("Error: {}",err)
                                            } else {
                                                format!("{}",
                                                    if let Some(c)=contents{c}else{String::new()}
                                                )
                                            }
                                        }),
                                        range:None
                                    }),
                                );
                            } else {
                                resp=RawResponse::ok::<HoverRequest>(
                                    id,
                                    &Some(Hover {
                                        contents:HoverContents::Markup(MarkupContent {
                                            kind:MarkupKind::PlainText,
                                            value:String::from("Could not access files"),
                                        }),
                                        range:None
                                    }),
                                );
                            }
                            //log.write(b"Writing response...\n").unwrap();
                            if let Ok(_)=sender.send(RawMessage::Response(resp)) {
                                //log.write(b"Sent response\n").unwrap();
                            } else {
                                //log.write(b"Failed to send response\n").unwrap();
                            }
                            //log.write(b"Wrote response!\n").unwrap();
                        } else {
                            //log.write(b"Not a hover request\n").unwrap();
                            // TODO: Add more stuff to here in the `} else if let
                            // Ok(stuff)=r.cast::<Type>() {}` format
                        }
                    },
                };
            },
            RawMessage::Response(_resp)=>{},
            RawMessage::Notification(not)=>{
                let change:bool;
                let mut name:String=String::new();
                let mut errors:Vec<NasmError>=Vec::new();
                if let Ok(params)=not.clone().cast::<DidOpenTextDocument>() {
                    let text_doc=params.text_document;
                    name=text_doc.uri.path().to_string();
                    let text=text_doc.text;
                    let mut files=FILES.lock().unwrap();
                    if let Some(nasm_file)=files.get_mut(&name) {
                        nasm_file.update_contents(text);
                        nasm_file.parse();
                        errors=nasm_file.errors.clone();
                    } else {
                        let mut nasm_file=NasmFile::new();
                        nasm_file.update_contents(text);
                        nasm_file.parse();
                        errors=nasm_file.errors.clone();
                        files.insert(name.to_string(),nasm_file);
                    }
                    change=true;
                } else if let Ok(params)=not.cast::<DidChangeTextDocument>() {
                    let text_doc=params.text_document;
                    name=text_doc.uri.path().to_string();
                    let text=params.content_changes[0].text.clone();
                    let mut files=FILES.lock().unwrap();
                    if let Some(nasm_file)=files.get_mut(&name) {
                        nasm_file.update_contents(text);
                        nasm_file.parse();
                        errors=nasm_file.errors.clone();
                    } else {
                        let mut nasm_file=NasmFile::new();
                        nasm_file.update_contents(text);
                        nasm_file.parse();
                        errors=nasm_file.errors.clone();
                        files.insert(name.to_string(),nasm_file);
                    }
                    change=true;
                } else {
                    change=false;
                }
                if change {
                    let mut diag:Vec<Diagnostic>=Vec::new();
                    for err in errors.clone() {
                        let severity=match err.error_type {
                            ErrorType::Warning=>Some(DiagnosticSeverity::Warning),
                            ErrorType::Error=>Some(DiagnosticSeverity::Error),
                            _=>None,
                        };
                        diag.push(Diagnostic {
                            range:Range::new(Position::new(err.line as u64-1,0),Position::new(err.line as u64-1,10)),
                            severity,
                            code:None,
                            source:None,
                            message:err.contents,
                            related_information:None,
                        });
                    }
                    let resp=RawNotification::new::<PublishDiagnostics>(
                        &PublishDiagnosticsParams {
                            uri:Url::from_file_path(name).unwrap(),
                            diagnostics:diag,
                        }
                    );
                    sender.send(RawMessage::Notification(resp));
                }
            },
        }
    }
    Ok(())
}
