mod nasm;


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
use nasm::*;


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
    let mut file=String::new();
    let mut errors:Vec<NasmError>=Vec::new();
    for msg in receiver {
        match msg {
            RawMessage::Request(req)=>{
                match handle_shutdown(req.clone(), sender) {
                    None=>return Ok(()),
                    Some(r)=>{
                        if let Ok((id,params))=r.clone().cast::<HoverRequest>() {
                            let mut contents:Option<String>=None;
                            let line=params.position.line;
                            for error in errors.clone() {
                                if error.line-1==line as usize {
                                    contents=Some(format!("{:?}: {}",error.error_type,error.contents));
                                    break;
                                }
                            }
                            sender.send(
                                RawMessage::Response(
                                    RawResponse::ok::<HoverRequest>(
                                        id,
                                        &Some(Hover {
                                            contents:HoverContents::Markup(MarkupContent {
                                                kind:MarkupKind::PlainText,
                                                value:format!("{}",if let Some(c)=contents{c}else{String::new()})
                                            }),
                                            range:None
                                        }),
                                    )
                                )
                            ).unwrap();
                        }
                    },
                };
            },
            RawMessage::Response(_resp)=>{},
            RawMessage::Notification(not)=>{
                let uri:Url;
                if let Ok(params)=not.clone().cast::<DidOpenTextDocument>() {
                    let text=params.text_document.text;
                    uri=params.text_document.uri;
                    file=text;
                } else if let Ok(params)=not.cast::<DidChangeTextDocument>() {
                    let changes=params.content_changes.len();
                    let text=params.content_changes[changes-1].text.clone();
                    uri=params.text_document.uri;
                    file=text;
                } else {continue}
                errors=Nasm::errors(file.clone()).unwrap();
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
                        uri,
                        diagnostics:diag,
                    }
                );
                sender.send(RawMessage::Notification(resp)).unwrap();
            },
        }
    }
    Ok(())
}
