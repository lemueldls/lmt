use std::{collections::HashMap, fs, ops::ControlFlow, path::PathBuf, sync::LazyLock};

use async_lsp::{
    ClientSocket, LanguageClient, LanguageServer, ResponseError,
    lsp_types::{
        CompletionOptions, Diagnostic, DidChangeConfigurationParams, DidChangeTextDocumentParams,
        DidOpenTextDocumentParams, DocumentFilter, ExecuteCommandOptions, GotoDefinitionParams,
        GotoDefinitionResponse, Hover, HoverContents, HoverParams, HoverProviderCapability,
        InitializeParams, InitializeResult, InlayHint, InlayHintKind, InlayHintLabel,
        InlayHintLabelPart, InlayHintParams, Location, MarkedString, OneOf, Position,
        PublishDiagnosticsParams, Range, ReferenceParams, SemanticToken, SemanticTokens,
        SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
        SemanticTokensParams, SemanticTokensRangeParams, SemanticTokensRangeResult,
        SemanticTokensRegistrationOptions, SemanticTokensResult, SemanticTokensServerCapabilities,
        ServerCapabilities, StaticRegistrationOptions, TextDocumentRegistrationOptions,
        TextDocumentSyncCapability, TextDocumentSyncKind, Url, WorkDoneProgressOptions,
        WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
        notification::Notification,
    },
    router::Router,
};
use futures::{future::BoxFuture, lock::Mutex};
use lmt_parser::{Input, Parser, SimpleSpan, error::RichReason};
use lmt_synthesis::{Graph, ModuleSynthesis, StaticSynthesis};
use ropey::Rope;

use crate::semantic_token::{ImCompleteSemanticToken, LEGEND_TYPE};

#[derive(Debug, Default)]
pub struct LspGraph {
    // pub document_map: HashMap<Url, Rope>,
}

impl Graph for LspGraph {
    fn resolve_path(&mut self, path: &str) -> Option<String> {
        todo!()
    }
}

pub static SYNTHESIS: LazyLock<StaticSynthesis> = LazyLock::new(StaticSynthesis::default);

#[derive(Debug)]
pub struct Backend {
    client: ClientSocket,
    synthesis_map: HashMap<Url, ModuleSynthesis>,
    document_map: HashMap<Url, Rope>,
    semantic_token_map: HashMap<Url, Vec<ImCompleteSemanticToken>>,
}

impl LanguageServer for Backend {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<async_lsp::Result<()>>;

    fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> BoxFuture<'static, Result<InitializeResult, Self::Error>> {
        eprintln!("Initialize with {params:?}");
        Box::pin(async move {
            Ok(InitializeResult {
                capabilities: ServerCapabilities {
                    inlay_hint_provider: Some(OneOf::Left(true)),
                    text_document_sync: Some(TextDocumentSyncCapability::Kind(
                        TextDocumentSyncKind::FULL,
                    )),
                    completion_provider: Some(CompletionOptions {
                        resolve_provider: Some(false),
                        trigger_characters: Some(vec![".".to_string()]),
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                        all_commit_characters: None,
                        completion_item: None,
                    }),
                    execute_command_provider: Some(ExecuteCommandOptions {
                        commands: vec!["dummy.do_something".to_string()],
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    }),
                    workspace: Some(WorkspaceServerCapabilities {
                        workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                            supported: Some(true),
                            change_notifications: Some(OneOf::Left(true)),
                        }),
                        file_operations: None,
                    }),
                    semantic_tokens_provider: Some(
                        SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                            SemanticTokensRegistrationOptions {
                                text_document_registration_options: {
                                    TextDocumentRegistrationOptions {
                                        document_selector: Some(vec![DocumentFilter {
                                            language: Some("lmt".to_string()),
                                            scheme: Some("file".to_string()),
                                            pattern: None,
                                        }]),
                                    }
                                },
                                semantic_tokens_options: SemanticTokensOptions {
                                    work_done_progress_options: WorkDoneProgressOptions::default(),
                                    legend: SemanticTokensLegend {
                                        token_types: LEGEND_TYPE.to_vec(),
                                        token_modifiers: vec![],
                                    },
                                    range: Some(true),
                                    full: Some(SemanticTokensFullOptions::Bool(true)),
                                },
                                static_registration_options: StaticRegistrationOptions::default(),
                            },
                        ),
                    ),
                    hover_provider: Some(HoverProviderCapability::Simple(true)),
                    definition_provider: Some(OneOf::Left(true)),
                    references_provider: Some(OneOf::Left(true)),
                    rename_provider: Some(OneOf::Left(true)),
                    ..ServerCapabilities::default()
                },
                server_info: None,
            })
        })
    }

    fn hover(
        &mut self,
        params: HoverParams,
    ) -> BoxFuture<'static, Result<Option<Hover>, Self::Error>> {
        let synthesis = self
            .synthesis_map
            .get(&params.text_document_position_params.text_document.uri);

        let uri = params.text_document_position_params.text_document.uri;
        let rope = self.document_map.get(&uri).unwrap();

        let position = params.text_document_position_params.position;
        let offset = rope.line_to_char(position.line as usize) + position.character as usize;

        let hover = synthesis.map(|synthesis| {
            let types = synthesis
                .ident_types
                .iter()
                .find(|(span, _)| span.into_range().contains(&offset));

            types.map(|(span, r#type)| {
                let start = offset_to_position(span.start, rope).unwrap();
                let end = offset_to_position(span.end, rope).unwrap();

                let mut proof_block = String::new();

                let proof = &r#type.borrow().proof;
                let equal_to = &proof.equal_to().unwrap();

                if let Some(upcast) = equal_to.upcast() {
                    proof_block += &format!("{proof}\nas {upcast}");
                } else {
                    proof_block += &format!("{proof}");
                }

                let mut implications_block = String::new();

                for implication in &r#type.borrow().implications {
                    let span = implication.for_type.span();

                    implications_block.push_str(&format!(
                        " | {} => {} is {}\n",
                        implication.if_equal,
                        rope.slice(span.start..span.end),
                        implication.then_implies,
                    ));
                }

                let mut contents = vec![MarkedString::from_language_code(
                    "lmt".to_string(),
                    proof_block,
                )];

                if !implications_block.is_empty() {
                    contents.push(MarkedString::from_language_code(
                        "lmt".to_string(),
                        implications_block,
                    ));
                }

                Hover {
                    contents: HoverContents::Array(contents),
                    range: Some(Range { start, end }),
                }
            })
        });

        let hover = Ok(hover.flatten());

        Box::pin(async move { hover })
    }

    fn definition(
        &mut self,
        params: GotoDefinitionParams,
    ) -> BoxFuture<'static, Result<Option<GotoDefinitionResponse>, ResponseError>> {
        let synthesis = self
            .synthesis_map
            .get(&params.text_document_position_params.text_document.uri);

        let uri = params.text_document_position_params.text_document.uri;
        let rope = self.document_map.get(&uri).unwrap();

        let position = params.text_document_position_params.position;
        let offset = rope.line_to_char(position.line as usize) + position.character as usize;

        let definition = synthesis.map(|synthesis| {
            let (_, def_span) = synthesis
                .ident_definitions
                .iter()
                .find(|(reference, _)| reference.into_range().contains(&offset))
                .unzip();

            def_span.map(|span| {
                let start = offset_to_position(span.start, rope).unwrap();
                let end = offset_to_position(span.end, rope).unwrap();

                GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: Range { start, end },
                })
            })
        });

        let definition = Ok(definition.flatten());

        Box::pin(async move { definition })
    }

    fn references(
        &mut self,
        params: ReferenceParams,
    ) -> BoxFuture<'static, Result<Option<Vec<Location>>, ResponseError>> {
        let synthesis = self
            .synthesis_map
            .get(&params.text_document_position.text_document.uri);

        let uri = params.text_document_position.text_document.uri;
        let rope = self.document_map.get(&uri).unwrap();

        let position = params.text_document_position.position;
        let offset = rope.line_to_char(position.line as usize) + position.character as usize;

        let references = synthesis.map(|synthesis| {
            let (_, references) = synthesis
                .ident_references
                .iter()
                .find(|(span, _)| span.into_range().contains(&offset))?;

            let locations = references
                .iter()
                .map(|span| {
                    let start = offset_to_position(span.start, rope).unwrap();
                    let end = offset_to_position(span.end, rope).unwrap();

                    Location {
                        uri: uri.clone(),
                        range: Range { start, end },
                    }
                })
                .collect();

            Some(locations)
        });

        let references = Ok(references.flatten());

        Box::pin(async move { references })
    }

    fn did_change_configuration(
        &mut self,
        _: DidChangeConfigurationParams,
    ) -> ControlFlow<async_lsp::Result<()>> {
        ControlFlow::Continue(())
    }

    fn did_open(
        &mut self,
        params: DidOpenTextDocumentParams,
    ) -> ControlFlow<async_lsp::Result<()>> {
        self.on_change(
            &params.text_document.text,
            params.text_document.uri,
            params.text_document.version,
        );

        ControlFlow::Continue(())
    }

    fn did_change(
        &mut self,
        params: DidChangeTextDocumentParams,
    ) -> ControlFlow<async_lsp::Result<()>> {
        self.on_change(
            &params.content_changes[0].text,
            params.text_document.uri,
            params.text_document.version,
        );

        ControlFlow::Continue(())
    }

    fn inlay_hint(
        &mut self,
        params: InlayHintParams,
    ) -> BoxFuture<'static, Result<Option<Vec<InlayHint>>, ResponseError>> {
        let uri = &params.text_document.uri;

        let inlay_hints = self.synthesis_map.get(uri).map(|synthesis| {
            synthesis
                .ident_references
                .keys()
                .map(|span| {
                    let (start, end) = self.span_to_pos(span, uri);

                    let r#type = synthesis.ident_types.get(span).unwrap();
                    let proof = &r#type.borrow().proof;
                    let equal_to = proof.equal_to().unwrap();

                    InlayHint {
                        text_edits: None,
                        tooltip: None,
                        kind: Some(InlayHintKind::TYPE),
                        padding_left: None,
                        padding_right: None,
                        data: None,
                        position: end,
                        label: InlayHintLabel::LabelParts(vec![InlayHintLabelPart {
                            value: format!(": {}", equal_to.upcast().as_ref().unwrap_or(equal_to)),
                            tooltip: None,
                            location: Some(Location {
                                uri: params.text_document.uri.clone(),
                                range: Range { start, end },
                            }),
                            command: None,
                        }]),
                    }
                })
                .chain(synthesis.functions.iter().map(|(span, function)| {
                    let (start, end) = self.span_to_pos(span, uri);

                    let proof = &function.return_type.borrow().proof;
                    let equal_to = proof.equal_to().unwrap();

                    InlayHint {
                        text_edits: None,
                        tooltip: None,
                        kind: Some(InlayHintKind::TYPE),
                        padding_left: None,
                        padding_right: None,
                        data: None,
                        position: end,
                        label: InlayHintLabel::LabelParts(vec![InlayHintLabelPart {
                            value: format!(": {}", equal_to.upcast().as_ref().unwrap_or(equal_to)),
                            tooltip: None,
                            location: Some(Location {
                                uri: params.text_document.uri.clone(),
                                range: Range { start, end },
                            }),
                            command: None,
                        }]),
                    }
                }))
                .collect()
        });

        Box::pin(async move { Ok(inlay_hints) })
    }

    fn semantic_tokens_full(
        &mut self,
        params: SemanticTokensParams,
    ) -> BoxFuture<'static, Result<Option<SemanticTokensResult>, ResponseError>> {
        let uri = params.text_document.uri;
        let semantic_tokens = || -> Option<Vec<SemanticToken>> {
            let im_complete_tokens = self.semantic_token_map.get(&uri)?;
            let rope = self.document_map.get(&uri)?;

            let mut pre_line = 0;
            let mut pre_start = 0;

            let semantic_tokens = im_complete_tokens
                .iter()
                .filter_map(|token| {
                    let line = u32::try_from(rope.try_byte_to_line(token.start).ok()?).ok()?;
                    let first = u32::try_from(rope.try_line_to_char(line as usize).ok()?).ok()?;
                    let start =
                        u32::try_from(rope.try_byte_to_char(token.start).ok()?).ok()? - first;
                    let ret = Some(SemanticToken {
                        delta_line: line - pre_line,
                        delta_start: if start >= pre_start {
                            start - pre_start
                        } else {
                            start
                        },
                        length: u32::try_from(token.length).ok()?,
                        token_type: u32::try_from(token.token_type).ok()?,
                        token_modifiers_bitset: 0,
                    });
                    pre_line = line;
                    pre_start = start;
                    ret
                })
                .collect::<Vec<_>>();
            Some(semantic_tokens)
        }();

        Box::pin(async move {
            match semantic_tokens {
                Some(semantic_tokens) => {
                    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                        result_id: None,
                        data: semantic_tokens,
                    })))
                }
                None => Ok(None),
            }
        })
    }

    fn semantic_tokens_range(
        &mut self,
        params: SemanticTokensRangeParams,
    ) -> BoxFuture<'static, Result<Option<SemanticTokensRangeResult>, ResponseError>> {
        let uri = params.text_document.uri;
        let semantic_tokens = || -> Option<Vec<SemanticToken>> {
            let im_complete_tokens = self.semantic_token_map.get(&uri)?;
            let rope = self.document_map.get(&uri)?;
            let mut pre_line = 0;
            let mut pre_start = 0;
            let semantic_tokens = im_complete_tokens
                .iter()
                .filter_map(|token| {
                    let line = u32::try_from(rope.try_byte_to_line(token.start).ok()?).ok()?;
                    let first = u32::try_from(rope.try_line_to_char(line as usize).ok()?).ok()?;
                    let start =
                        u32::try_from(rope.try_byte_to_char(token.start).ok()?).ok()? - first;
                    let ret = Some(SemanticToken {
                        delta_line: line - pre_line,
                        delta_start: if start >= pre_start {
                            start - pre_start
                        } else {
                            start
                        },
                        length: u32::try_from(token.length).ok()?,
                        token_type: u32::try_from(token.token_type).ok()?,
                        token_modifiers_bitset: 0,
                    });
                    pre_line = line;
                    pre_start = start;
                    ret
                })
                .collect::<Vec<_>>();
            Some(semantic_tokens)
        }();

        Box::pin(async move {
            if let Some(semantic_tokens) = semantic_tokens {
                Ok(Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
                    result_id: None,
                    data: semantic_tokens,
                })))
            } else {
                Ok(None)
            }
        })
    }
}

enum CustomNotification {}

impl Notification for CustomNotification {
    type Params = InlayHintParams;

    const METHOD: &'static str = "custom/notification";
}

impl Backend {
    fn on_change(&mut self, text: &str, uri: Url, version: i32) {
        let rope = ropey::Rope::from_str(text);
        self.document_map.insert(uri.clone(), rope.clone());

        let (module_id, parse_errors) = SYNTHESIS.load_module("examples/test.lmt", text);
        let synthesis = SYNTHESIS.get_module_synthesis(module_id);

        // let semantic_tokens = ast
        //         .as_ref()
        //         .map(crate::semantic_token::semantic_token_from_ast);

        // let parse_errors = tokenize_errors
        //     .into_iter()
        //     .map(|err| err.map_token(|ch| ch.to_string()))
        //     .chain(
        //         synthesis_errors
        //             .into_iter()
        //             .map(|err| err.map_token(|token| token.to_string())),
        //     );

        // let parse_diagnostics = parse_errors.into_iter().filter_map(|item| {
        //     let (message, span) = match item.reason() {
        //         RichReason::ExpectedFound { expected, found } => {
        //             (
        //                 format!(
        //                     "Expected {}, found {}",
        //                     if expected.is_empty() {
        //                         "something else".to_string()
        //                     } else {
        //                         expected
        //                             .iter()
        //                             .map(std::string::ToString::to_string)
        //                             .collect::<Vec<_>>()
        //                             .join(", ")
        //                     },
        //                     if let Some(found) = found {
        //                         if found.len() == 0 {
        //                             "something else".to_string()
        //                         } else {
        //                             found.to_string()
        //                         }
        //                     } else {
        //                         "nothing".to_string()
        //                     }
        //                 ),
        //                 item.span(),
        //             )
        //         }
        //         RichReason::Custom(reason) => (reason.to_string(), item.span()),
        //         RichReason::Many(reasons) => {
        //             let mut message = String::new();
        //             for reason in reasons {
        //                 match reason {
        //                     RichReason::ExpectedFound { expected, found } => {
        //                         message.push_str(&format!(
        //                             "Expected {}, found {}",
        //                             expected
        //                                 .iter()
        //                                 .map(std::string::ToString::to_string)
        //                                 .collect::<Vec<_>>()
        //                                 .join(", "),
        //                             found
        //                                 .iter()
        //                                 .map(|item| item.to_string())
        //                                 .collect::<Vec<_>>()
        //                                 .join(", ")
        //                         ));
        //                     }
        //                     RichReason::Custom(reason) => {
        //                         message.push_str(&reason.to_string());
        //                     }
        //                     _ => {}
        //                 }
        //             }
        //             (message, item.span())
        //         }
        //     };

        //     let start_position = offset_to_position(span.start, &rope)?;
        //     let end_position = offset_to_position(span.end, &rope)?;

        //     Some(Diagnostic::new_simple(
        //         Range::new(start_position, end_position),
        //         message,
        //     ))
        // });

        // let synthesis_diagnostics = synthesis.iter().flat_map(|synthesis| {
        //     synthesis.errors.iter().filter_map(|error| {
        //         let span = error.span();
        //         let start_position = offset_to_position(span.start, &rope)?;
        //         let end_position = offset_to_position(span.end, &rope)?;

        //         Some(Diagnostic::new_simple(
        //             Range::new(start_position, end_position),
        //             error.to_string(),
        //         ))
        //     })
        // });

        // let diagnostics = parse_diagnostics.chain(synthesis_diagnostics).collect();

        // self.client
        //     .publish_diagnostics(PublishDiagnosticsParams {
        //         uri: uri.clone(),
        //         diagnostics,
        //         version: Some(version),
        //     })
        //     .unwrap();

        // if let (Some(synthesis), Some(semantic_tokens)) = (synthesis, semantic_tokens) {
        //     self.synthesis_map.insert(uri.clone(), synthesis);
        //     self.semantic_token_map.insert(uri, semantic_tokens);
        // }
    }

    fn span_to_pos(&self, span: &SimpleSpan, uri: &Url) -> (Position, Position) {
        let rope = &self.document_map[uri];
        let start = offset_to_position(span.start, rope).unwrap_or_else(|| Position::new(0, 0));
        let end = offset_to_position(span.end, rope).unwrap_or_else(|| Position::new(0, 0));

        (start, end)
    }
}

impl Backend {
    pub fn new_router(client: ClientSocket) -> Router<Self> {
        let state = Self {
            client,
            synthesis_map: HashMap::new(),
            document_map: HashMap::new(),
            semantic_token_map: HashMap::new(),
        };

        let mut router = Router::from_language_server(state);
        router.event(Self::on_tick);
        router
    }

    pub fn on_tick(&mut self, (): ()) -> ControlFlow<async_lsp::Result<()>> {
        // info!("tick");
        // self.counter += 1;
        ControlFlow::Continue(())
    }
}

fn offset_to_position(offset: usize, rope: &Rope) -> Option<Position> {
    let line = rope.try_char_to_line(offset).ok()?;
    let first_char_of_line = rope.try_line_to_char(line).ok()?;
    let column = offset - first_char_of_line;

    Some(Position::new(
        u32::try_from(line).ok()?,
        u32::try_from(column).ok()?,
    ))
}
