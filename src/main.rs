use std::{borrow::Cow, collections::HashMap, fmt::Display, net::Ipv4Addr, sync::Arc};

use phf::phf_set;
use serde_json::Value;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tower_lsp::{
    async_trait,
    jsonrpc::Result,
    lsp_types::{
        CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams,
        CodeActionProviderCapability, CompletionItem, CompletionItemKind,
        CompletionItemLabelDetails, CompletionOptions, CompletionOptionsCompletionItem,
        CompletionParams, CompletionResponse, CompletionTextEdit, Diagnostic,
        DiagnosticRelatedInformation, DiagnosticSeverity, DidChangeConfigurationParams,
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, DocumentSymbolParams,
        DocumentSymbolResponse, Documentation, ExecuteCommandOptions, ExecuteCommandParams,
        GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents, HoverParams,
        HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, InlayHint,
        InlayHintKind, InlayHintLabel, InlayHintParams, LanguageString, Location, MarkedString,
        MessageType, NumberOrString, OneOf, ParameterInformation, ParameterLabel,
        Position as LspPosition, PositionEncodingKind, Range as LspRange, SemanticToken,
        SemanticTokenType, SemanticTokens, SemanticTokensFullOptions, SemanticTokensLegend,
        SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult,
        SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo, SignatureHelp,
        SignatureHelpOptions, SignatureHelpParams, SignatureInformation, SymbolInformation,
        SymbolKind, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Url,
        WorkDoneProgressOptions, WorkspaceEdit,
    },
    Client, LanguageServer, LspService, Server,
};
#[cfg(not(target_arch = "wasm32"))]
use tree_sitter::{Node, Parser, Query, QueryCursor, StreamingIterator as _, Tree};
#[cfg(target_arch = "wasm32")]
use tree_sitter_c2rust::{Node, Parser, Query, QueryCursor, StreamingIterator as _, Tree};

mod cli;
mod instructions;

const LINT_ABSOLUTE_JUMP: &'static str = "L001";
const LINT_NUMBER_BATCH_MODE: &'static str = "L002";
const LINT_NUMBER_REAGENT_MODE: &'static str = "L003";

const SEMANTIC_SYMBOL_LEGEND: &'static [SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,
    SemanticTokenType::COMMENT,
    SemanticTokenType::STRING,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::MACRO,
    SemanticTokenType::NUMBER,
    SemanticTokenType::VARIABLE,
];
struct DocumentData {
    url: Url,
    content: String,
    tree: Option<Tree>,
    parser: Parser,
}

#[derive(Debug)]
struct DefinitionData<T> {
    range: Range,
    value: T,
}

impl<T> DefinitionData<T> {
    fn new(range: Range, value: T) -> Self {
        DefinitionData { range, value }
    }
}

#[derive(Debug)]
enum AliasValue {
    Register(String),
    Device(String),
}

impl Display for AliasValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AliasValue::Register(s) => s,
            AliasValue::Device(s) => s,
        };
        s.fmt(f)
    }
}

impl From<String> for AliasValue {
    fn from(value: String) -> Self {
        use AliasValue::*;
        if value.starts_with("d") {
            Device(value)
        } else {
            Register(value)
        }
    }
}

trait HasType {
    fn get_type(&self) -> instructions::DataType;
}

impl HasType for AliasValue {
    fn get_type(&self) -> instructions::DataType {
        match *self {
            AliasValue::Register(_) => instructions::DataType::Register,
            AliasValue::Device(_) => instructions::DataType::Device,
        }
    }
}

impl HasType for DefinitionData<f64> {
    fn get_type(&self) -> instructions::DataType {
        instructions::DataType::Number
    }
}

impl HasType for DefinitionData<String> {
    fn get_type(&self) -> instructions::DataType {
        instructions::DataType::Number
    }
}

impl HasType for DefinitionData<u8> {
    fn get_type(&self) -> instructions::DataType {
        instructions::DataType::Number
    }
}

impl<T> HasType for DefinitionData<T>
where
    T: HasType,
{
    fn get_type(&self) -> instructions::DataType {
        self.value.get_type()
    }
}

#[derive(Default, Debug)]
struct TypeData {
    defines: HashMap<String, DefinitionData<String>>,
    aliases: HashMap<String, DefinitionData<AliasValue>>,
    labels: HashMap<String, DefinitionData<u8>>,
}

impl TypeData {
    fn get_range(&self, name: &str) -> Option<Range> {
        if let Some(definition_data) = self.defines.get(name) {
            return Some(definition_data.range.clone());
        }
        if let Some(definition_data) = self.aliases.get(name) {
            return Some(definition_data.range.clone());
        }
        if let Some(definition_data) = self.labels.get(name) {
            return Some(definition_data.range.clone());
        }
        None
    }
}

struct FileData {
    document_data: DocumentData,
    type_data: TypeData,
}

#[derive(Clone, Debug)]
struct Configuration {
    max_lines: usize,
    max_columns: usize,
    warn_overline_comment: bool,
    warn_overcolumn_comment: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            max_lines: 128,
            max_columns: 90, //lines can be 90 characters long these days
            warn_overline_comment: true,
            warn_overcolumn_comment: false,
        }
    }
}

struct Backend {
    client: Client,
    files: Arc<RwLock<HashMap<Url, FileData>>>,
    config: Arc<RwLock<Configuration>>,
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let mut utf8_supported = false;
        if let Some(encodings) = params
            .capabilities
            .general
            .and_then(|x| x.position_encodings)
        {
            for encoding in encodings {
                if encoding == PositionEncodingKind::UTF8 {
                    utf8_supported = true;
                }
            }
            if !utf8_supported {
                self.client
                    .show_message(
                        MessageType::WARNING,
                        "Client does not support UTF-8. Non-ASCII characters will cause problems.",
                    )
                    .await;
            }
        }
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["version".to_string()],
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),
                inlay_hint_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec![" ".to_string()]),
                    retrigger_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                position_encoding: utf8_supported.then_some(PositionEncodingKind::UTF8),
                document_symbol_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![" ".to_string()]),
                    completion_item: Some(CompletionOptionsCompletionItem {
                        label_details_support: Some(true),
                    }),
                    ..Default::default()
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            range: Some(false),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            legend: {
                                SemanticTokensLegend {
                                    token_types: SEMANTIC_SYMBOL_LEGEND.into(),
                                    token_modifiers: vec![],
                                }
                            },
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "ic10lsp".to_string(),
                version: Some("1.0.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {}

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        if params.command == "version" {
            self.client
                .show_message(
                    MessageType::INFO,
                    concat!("IC10LSP Version: ", env!("CARGO_PKG_VERSION")),
                )
                .await;
        }
        Ok(None)
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.update_content(params.text_document.uri.clone(), params.text_document.text)
            .await;
        self.run_diagnostics(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        for change in params.content_changes {
            // Should only ever be one, because we are getting full updates
            self.update_content(params.text_document.uri.clone(), change.text)
                .await;
        }
        self.run_diagnostics(&params.text_document.uri).await;
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        {
            let mut config = self.config.write().await;
            let value = params.settings;

            if let Some(warnings) = value.get("warnings").and_then(Value::as_object) {
                config.warn_overline_comment = warnings
                    .get("overline_comment")
                    .and_then(Value::as_bool)
                    .unwrap_or(config.warn_overline_comment);

                config.warn_overcolumn_comment = warnings
                    .get("overcolumn_comment")
                    .and_then(Value::as_bool)
                    .unwrap_or(config.warn_overcolumn_comment);
            }

            config.max_lines = value
                .get("max_lines")
                .and_then(Value::as_u64)
                .map(|x| x as usize)
                .unwrap_or(config.max_lines);

            config.max_columns = value
                .get("max_columns")
                .and_then(Value::as_u64)
                .map(|x| x as usize)
                .unwrap_or(config.max_columns);
        }

        let uris = {
            let files = self.files.read().await;
            files.keys().map(Clone::clone).collect::<Vec<_>>()
        };
        for uri in uris {
            self.run_diagnostics(&uri).await;
        }
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let mut ret = Vec::new();

        let files = self.files.read().await;
        let uri = params.text_document.uri;
        let Some(file_data) = files.get(&uri) else {
            return Err(tower_lsp::jsonrpc::Error::invalid_request());
        };

        let document = &file_data.document_data;

        let Some(ref tree) = document.tree else {
            return Err(tower_lsp::jsonrpc::Error::internal_error());
        };

        let mut cursor = QueryCursor::new();
        let query = Query::new(&tree_sitter_ic10::language(), "(number)@x").unwrap();

        let mut captures = cursor.captures(&query, tree.root_node(), document.content.as_bytes());
        while let Some((capture, _)) = captures.next() {
            let node = capture.captures[0].node;

            let range = Range::from(node.range());
            if !range.contains(node.range().start_point.into())
                || !range.contains(node.range().end_point.into())
            {
                continue;
            }

            let text = node.utf8_text(document.content.as_bytes()).unwrap();
            if let Some(item_name) = instructions::HASH_NAME_LOOKUP.get(text) {
                let Some(line_node) = node.find_parent("line") else {
                    continue;
                };

                let endpos = if let Some(newline) =
                    line_node.query("(newline)@x", document.content.as_bytes())
                {
                    Position::from(newline.range().start_point)
                } else if let Some(instruction) =
                    line_node.query("(instruction)@x", document.content.as_bytes())
                {
                    Position::from(instruction.range().end_point)
                } else {
                    Position::from(node.range().end_point)
                };

                ret.push(InlayHint {
                    position: endpos.into(),
                    label: InlayHintLabel::String(item_name.to_string()),
                    kind: Some(InlayHintKind::TYPE),
                    text_edits: None,
                    tooltip: None,
                    padding_left: None,
                    padding_right: None,
                    data: None,
                });
            }
        }

        Ok(Some(ret))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let mut ret = Vec::new();
        let files = self.files.read().await;
        let uri = params.text_document.uri;
        let Some(file_data) = files.get(&uri) else {
            return Err(tower_lsp::jsonrpc::Error::invalid_request());
        };
        let document = &file_data.document_data;

        let Some(ref tree) = document.tree else {
            return Err(tower_lsp::jsonrpc::Error::internal_error());
        };

        let mut cursor = QueryCursor::new();
        let query = Query::new(
            &tree_sitter_ic10::language(),
            "(comment) @comment
             (instruction (operation)@keyword)
             (logictype)@string
             (device)@preproc
             (register)@macro
             (number)@float
             (identifier)@variable",
        )
        .unwrap();

        let mut previous_line = 0u32;
        let mut previous_col = 0u32;

        let comment_idx = query.capture_index_for_name("comment").unwrap();
        let keyword_idx = query.capture_index_for_name("keyword").unwrap();
        let string_idx = query.capture_index_for_name("string").unwrap();
        let preproc_idx = query.capture_index_for_name("preproc").unwrap();
        let macro_idx = query.capture_index_for_name("macro").unwrap();
        let float_idx = query.capture_index_for_name("float").unwrap();
        let variable_idx = query.capture_index_for_name("variable").unwrap();

        let mut captures = cursor.captures(&query, tree.root_node(), document.content.as_bytes());
        while let Some((capture, _)) = captures.next() {
            let node = capture.captures[0].node;
            let idx = capture.captures[0].index;
            let start = node.range().start_point;

            let delta_line = start.row as u32 - previous_line;
            let delta_start = if delta_line == 0 {
                start.column as u32 - previous_col
            } else {
                start.column as u32
            };

            let tokentype = {
                if idx == comment_idx {
                    SemanticTokenType::COMMENT
                } else if idx == keyword_idx {
                    SemanticTokenType::KEYWORD
                } else if idx == string_idx {
                    SemanticTokenType::STRING
                } else if idx == preproc_idx {
                    SemanticTokenType::FUNCTION
                } else if idx == macro_idx {
                    SemanticTokenType::MACRO
                } else if idx == float_idx {
                    SemanticTokenType::NUMBER
                } else if idx == variable_idx {
                    SemanticTokenType::VARIABLE
                } else {
                    continue;
                }
            };

            ret.push(SemanticToken {
                delta_line,
                delta_start,
                length: node.range().end_point.column as u32 - start.column as u32,
                token_type: SEMANTIC_SYMBOL_LEGEND
                    .iter()
                    .position(|x| *x == tokentype)
                    .unwrap() as u32,
                token_modifiers_bitset: 0,
            });

            previous_line = start.row as u32;
            previous_col = start.column as u32;
        }
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: ret,
        })))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let mut ret = Vec::new();
        let files = self.files.read().await;
        let uri = params.text_document.uri;

        let Some(file_data) = files.get(&uri) else {
            return Err(tower_lsp::jsonrpc::Error::invalid_request());
        };

        let document = &file_data.document_data;

        let Some(ref tree) = document.tree else {
            return Err(tower_lsp::jsonrpc::Error::internal_error());
        };

        let mut cursor = QueryCursor::new();
        let query = Query::new(
            &tree_sitter_ic10::language(),
            "(instruction (operation \"define\") . (operand)@name)@define
            (instruction (operation \"alias\") . (operand)@name)@alias
            (instruction (operation \"label\") . (operand)@name)@alias
            (label (identifier)@name)@label",
        )
        .unwrap();
        let define_idx = query.capture_index_for_name("define").unwrap();
        let alias_idx = query.capture_index_for_name("alias").unwrap();
        let label_idx = query.capture_index_for_name("label").unwrap();
        let name_idx = query.capture_index_for_name("name").unwrap();

        let mut matches = cursor.matches(&query, tree.root_node(), document.content.as_bytes());

        while let Some(matched) = matches.next() {
            let main_match = {
                let mut ret = None;
                for cap in matched.captures {
                    if cap.index == define_idx || cap.index == alias_idx || cap.index == label_idx {
                        ret = Some(cap);
                    }
                }
                match ret {
                    Some(ret) => ret,
                    None => continue,
                }
            };

            let kind = if main_match.index == define_idx {
                SymbolKind::NUMBER
            } else if main_match.index == alias_idx {
                SymbolKind::VARIABLE
            } else if main_match.index == label_idx {
                SymbolKind::FUNCTION
            } else {
                SymbolKind::FILE
            };

            let Some(name_node) = matched.nodes_for_capture_index(name_idx).next() else {
                continue;
            };

            let name = name_node.utf8_text(document.content.as_bytes()).unwrap();
            #[allow(deprecated)]
            ret.push(SymbolInformation {
                name: name.to_string(),
                kind,
                tags: None,
                deprecated: Some(matched.pattern_index == 2),
                location: Location::new(uri.clone(), Range::from(name_node.range()).into()),
                container_name: None,
            });
        }
        Ok(Some(DocumentSymbolResponse::Flat(ret)))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        fn instruction_completions(prefix: &str, completions: &mut Vec<CompletionItem>) {
            let start_entries = completions.len();
            for (instruction, signature) in instructions::INSTRUCTIONS.entries() {
                if instruction.starts_with(prefix) {
                    completions.push(CompletionItem {
                        label: instruction.to_string(),
                        label_details: Some(CompletionItemLabelDetails {
                            detail: Some(format!("{signature}")),
                            description: None,
                        }),
                        kind: Some(CompletionItemKind::FUNCTION),
                        documentation: instructions::INSTRUCTION_DOCS
                            .get(instruction)
                            .map(|x| Documentation::String(x.to_string())),
                        deprecated: Some(*instruction == "label"),
                        ..Default::default()
                    });
                }
            }
            let length = completions.len();
            completions[start_entries..length].sort_by(|x, y| x.label.cmp(&y.label));
        }

        fn param_completions_static(
            prefix: &str,
            detail: &str,
            param_type: &instructions::Union,
            completions: &mut Vec<CompletionItem>,
        ) {
            use instructions::DataType;

            let start_entries = completions.len();

            for typ in param_type.0 {
                let map = match typ {
                    DataType::LogicType => instructions::LOGIC_TYPE_DOCS,
                    DataType::SlotLogicType => instructions::SLOT_TYPE_DOCS,
                    DataType::BatchMode => instructions::BATCH_MODE_DOCS,
                    _ => continue,
                };

                for entry in map.entries() {
                    let name = *entry.0;
                    let docs = *entry.1;
                    if name.starts_with(prefix) {
                        completions.push(CompletionItem {
                            label: name.to_string(),
                            label_details: Some(CompletionItemLabelDetails {
                                description: None,
                                detail: Some(detail.to_string()),
                            }),
                            kind: Some(CompletionItemKind::CONSTANT),
                            documentation: Some(Documentation::String(docs.to_string())),
                            ..Default::default()
                        });
                    }
                }
            }
            let length = completions.len();
            completions[start_entries..length].sort_by(|x, y| x.label.cmp(&y.label));
        }

        fn param_completions_dynamic<T>(
            prefix: &str,
            map: &HashMap<String, DefinitionData<T>>,
            detail: &str,
            param_type: &instructions::Union,
            completions: &mut Vec<CompletionItem>,
        ) where
            DefinitionData<T>: HasType,
            T: std::fmt::Display,
        {
            let start_entries = completions.len();
            for (identifier, value_data) in map.iter() {
                let value = &value_data.value;
                if identifier.starts_with(prefix) && param_type.match_type(value_data.get_type()) {
                    completions.push(CompletionItem {
                        label: identifier.to_string(),
                        label_details: Some(CompletionItemLabelDetails {
                            description: Some(format!("{value}")),
                            detail: Some(detail.to_string()),
                        }),
                        kind: Some(CompletionItemKind::VARIABLE),
                        ..Default::default()
                    });
                }
            }
            let length = completions.len();
            completions[start_entries..length].sort_by(|x, y| x.label.cmp(&y.label));
        }

        let mut ret = Vec::new();

        let uri = params.text_document_position.text_document.uri;
        let position = {
            let pos = params.text_document_position.position;
            Position::from(tower_lsp::lsp_types::Position::new(
                pos.line,
                pos.character.saturating_sub(1),
            ))
        };

        let files = self.files.read().await;
        let Some(file_data) = files.get(&uri) else {
            return Err(tower_lsp::jsonrpc::Error::invalid_request());
        };

        let document = &file_data.document_data;

        let Some(ref tree) = document.tree else {
            return Err(tower_lsp::jsonrpc::Error::internal_error());
        };

        let Some(node) = self.node_at_position(position, tree) else {
            return Ok(None);
        };

        if let Some(node) = node.find_parent("operation") {
            let text = node.utf8_text(document.content.as_bytes()).unwrap();
            let cursor_pos = position.0.character as usize - node.start_position().column;
            let prefix = &text[..cursor_pos + 1];

            instruction_completions(prefix, &mut ret);
        } else if let Some(node) = node.find_parent("invalid_instruction") {
            let text = node.utf8_text(document.content.as_bytes()).unwrap();
            let cursor_pos = position.0.character as usize - node.start_position().column;
            let prefix = &text[..cursor_pos + 1];

            instruction_completions(prefix, &mut ret);
        } else if let Some(line_node) = node.find_parent("line") {
            let text = line_node.utf8_text(document.content.as_bytes()).unwrap();
            let cursor_pos = position.0.character as usize - line_node.start_position().column;
            let global_prefix = &text[..cursor_pos as usize + 1];

            if global_prefix.chars().all(char::is_whitespace) {
                instruction_completions("", &mut ret);
            } else {
                let Some(line_node) = node.find_parent("line") else {
                    return Ok(None);
                };

                let Some(instruction_node) = line_node.query(
                    "(instruction)@x",
                    file_data.document_data.content.as_bytes(),
                ) else {
                    return Ok(None);
                };

                let Some(operation_node) = instruction_node.child_by_field_name("operation") else {
                    return Ok(None);
                };

                let text = operation_node
                    .utf8_text(file_data.document_data.content.as_bytes())
                    .unwrap();

                let (current_param, operand_node) =
                    get_current_parameter(instruction_node, position.0.character as usize);

                let operand_text = operand_node
                    .map(|node| node.utf8_text(document.content.as_bytes()).unwrap())
                    .unwrap_or("");

                let prefix = {
                    if let Some(operand_node) = operand_node {
                        let cursor_pos = (position.0.character as usize)
                            .saturating_sub(operand_node.start_position().column);
                        &operand_text[..(cursor_pos + 1).min(operand_text.len())]
                    } else {
                        ""
                    }
                };

                let Some(signature) = instructions::INSTRUCTIONS.get(text) else {
                    return Ok(None);
                };

                let Some(param_type) = signature.0.get(current_param) else {
                    return Ok(None);
                };

                if let Some(preproc_string_node) = instruction_node.query(
                    "(preproc_string)@x",
                    file_data.document_data.content.as_bytes(),
                ) {
                    let string_text = preproc_string_node
                        .utf8_text(file_data.document_data.content.as_bytes())
                        .unwrap();

                    let start_entries = ret.len();

                    for hash_name in &instructions::HASH_NAMES {
                        if hash_name.starts_with(string_text) {
                            ret.push(CompletionItem {
                                label: hash_name.to_string(),
                                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                                    range: {
                                        let mut edit_range =
                                            Range::from(preproc_string_node.range());
                                        edit_range.0.end.character -= 1;
                                        edit_range.into()
                                    },
                                    new_text: hash_name.to_string(),
                                })),
                                ..Default::default()
                            });
                        }
                    }
                    let length = ret.len();
                    ret[start_entries..length].sort_by(|x, y| x.label.cmp(&y.label));
                };

                if !text.starts_with("br") && text.starts_with("b") || text == "j" || text == "jal"
                {
                    param_completions_static(prefix, "", param_type, &mut ret);

                    param_completions_dynamic(
                        prefix,
                        &file_data.type_data.labels,
                        " label",
                        param_type,
                        &mut ret,
                    );

                    param_completions_dynamic(
                        prefix,
                        &file_data.type_data.defines,
                        " define",
                        param_type,
                        &mut ret,
                    );

                    param_completions_dynamic(
                        prefix,
                        &file_data.type_data.aliases,
                        " alias",
                        param_type,
                        &mut ret,
                    );
                } else {
                    param_completions_static(prefix, "", param_type, &mut ret);

                    param_completions_dynamic(
                        prefix,
                        &file_data.type_data.defines,
                        " define",
                        param_type,
                        &mut ret,
                    );

                    param_completions_dynamic(
                        prefix,
                        &file_data.type_data.aliases,
                        " alias",
                        param_type,
                        &mut ret,
                    );

                    param_completions_dynamic(
                        prefix,
                        &file_data.type_data.labels,
                        " label",
                        param_type,
                        &mut ret,
                    );
                }
            }
        }

        Ok(Some(CompletionResponse::Array(ret)))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = Position::from(params.text_document_position_params.position);

        let files = self.files.read().await;
        let Some(file_data) = files.get(&uri) else {
            return Err(tower_lsp::jsonrpc::Error::invalid_request());
        };

        let document = &file_data.document_data;

        let Some(ref tree) = document.tree else {
            return Err(tower_lsp::jsonrpc::Error::internal_error());
        };

        let Some(node) = self.node_at_position(position, tree) else {
            return Ok(None);
        };

        let Some(line_node) = node.find_parent("line") else {
            return Ok(None);
        };

        let Some(instruction_node) =
            line_node.query("(instruction)@x", document.content.as_bytes())
        else {
            return Ok(None);
        };

        let Some(operation_node) = instruction_node.child_by_field_name("operation") else {
            return Ok(None);
        };

        let text = operation_node
            .utf8_text(document.content.as_bytes())
            .unwrap();

        let (current_param, _) = get_current_parameter(
            instruction_node,
            position.0.character.saturating_sub(1) as usize,
        );

        let Some(signature) = instructions::INSTRUCTIONS.get(text) else {
            return Ok(None);
        };

        let mut label = text.to_string();
        let mut parameters = Vec::new();

        for parameter in signature.0 {
            let start = label.len() as u32 + 1;
            label.push_str(&format!(" {parameter}"));
            let end = label.len() as u32 + 0;
            parameters.push([start, end]);
        }

        Ok(Some(SignatureHelp {
            signatures: vec![SignatureInformation {
                label: label,
                documentation: instructions::INSTRUCTION_DOCS
                    .get(text)
                    .map(|x| Documentation::String(x.to_string())),
                parameters: Some(
                    parameters
                        .iter()
                        .map(|offset| ParameterInformation {
                            label: ParameterLabel::LabelOffsets(offset.to_owned()),
                            documentation: None,
                        })
                        .collect(),
                ),
                active_parameter: Some(current_param as u32),
            }],
            active_signature: None,
            active_parameter: None,
        }))
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<Vec<CodeActionOrCommand>>> {
        let mut ret = Vec::new();

        let files = self.files.read().await;
        let Some(file_data) = files.get(&params.text_document.uri) else {
            return Err(tower_lsp::jsonrpc::Error::invalid_request());
        };

        let document = &file_data.document_data;
        let uri = &document.url;

        let Some(ref tree) = document.tree else {
            return Err(tower_lsp::jsonrpc::Error::invalid_request());
        };

        let Some(node) = self.node_at_range(params.range.into(), tree) else {
            return Ok(None);
        };

        'diagnostics: for diagnostic in params.context.diagnostics {
            let Some(line_node) = node.find_parent("line") else {
                continue 'diagnostics;
            };

            let Some(NumberOrString::String(code)) = diagnostic.code.clone() else {
                continue;
            };
            match code.as_str() {
                LINT_NUMBER_BATCH_MODE => {
                    let replacement = diagnostic.data.as_ref().unwrap().as_str().unwrap();

                    let edit = TextEdit::new(diagnostic.range, replacement.to_string());

                    ret.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: format!("Replace with {replacement}"),
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: Some(vec![diagnostic]),
                        edit: Some(WorkspaceEdit::new(HashMap::from([(
                            uri.clone(),
                            vec![edit],
                        )]))),
                        is_preferred: Some(true),
                        ..Default::default()
                    }));
                }
                LINT_ABSOLUTE_JUMP => {
                    const REPLACEMENTS: phf::Map<&'static str, &'static str> = phf::phf_map! {
                        "bdns" => "brdns",
                        "bdse" => "brdse",
                        "bap" => "brap",
                        "bapz" => "brapz",
                        "beq" => "breq",
                        "beqz" => "breqz",
                        "bge" => "brge",
                        "bgez" => "brgez",
                        "bgt" => "brgt",
                        "bgtz" => "brgtz",
                        "ble" => "brle",
                        "blez" => "brlez",
                        "blt" => "brlt",
                        "bltz" => "brltz",
                        "bna" => "brna",
                        "bnaz" => "brnaz",
                        "bne" => "brne",
                        "bnez" => "brnez",
                        "j" => "jr",
                    };

                    if let Some(node) =
                        line_node.query("(instruction (operation)@x)", document.content.as_bytes())
                    {
                        let text = node.utf8_text(document.content.as_bytes()).unwrap();

                        if let Some(replacement) = REPLACEMENTS.get(text) {
                            let edit = TextEdit::new(
                                Range::from(node.range()).into(),
                                replacement.to_string(),
                            );

                            ret.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title: format!("Replace with {replacement}"),
                                kind: Some(CodeActionKind::QUICKFIX),
                                diagnostics: Some(vec![diagnostic]),
                                edit: Some(WorkspaceEdit::new(HashMap::from([(
                                    uri.clone(),
                                    vec![edit],
                                )]))),
                                command: None,
                                is_preferred: Some(true),
                                disabled: None,
                                data: None,
                            }));
                        }

                        break;
                    }
                }
                _ => {}
            }
        }

        Ok(Some(ret))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let files = self.files.read().await;
        let Some(file_data) = files.get(&params.text_document_position_params.text_document.uri)
        else {
            return Err(tower_lsp::jsonrpc::Error::internal_error());
        };
        let document = &file_data.document_data;
        let type_data = &file_data.type_data;

        let position = params.text_document_position_params.position;

        if let Some(tree) = document.tree.as_ref() {
            if let Some(node) = self.node_at_position(position.into(), tree) {
                if node.kind() == "identifier" {
                    let name = node.utf8_text(document.content.as_bytes()).unwrap();
                    if let Some(range) = type_data.get_range(name) {
                        return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                            document.url.clone(),
                            range.0,
                        ))));
                    }
                }
            }
        }
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let files = self.files.read().await;
        let Some(file_data) = files.get(&params.text_document_position_params.text_document.uri)
        else {
            return Err(tower_lsp::jsonrpc::Error::internal_error());
        };
        let document = &file_data.document_data;
        let type_data = &file_data.type_data;

        let position = params.text_document_position_params.position;

        let Some(tree) = document.tree.as_ref() else {
            return Ok(None);
        };
        let root = tree.root_node();
        let Some(node) = root.named_descendant_for_point_range(
            tree_sitter::Point::new(position.line as usize, position.character as usize),
            tree_sitter::Point::new(position.line as usize, position.character as usize + 1),
        ) else {
            return Ok(None);
        };

        let name = node.utf8_text(document.content.as_bytes()).unwrap();
        match node.kind() {
            "identifier" => {
                if let Some(definition_data) = type_data.defines.get(name) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Array(vec![MarkedString::LanguageString(
                            LanguageString {
                                language: "ic10".to_string(),
                                value: format!("define {} {}", name, definition_data.value),
                            },
                        )]),
                        range: Some(Range::from(node.range()).into()),
                    }));
                }
                if let Some(definition_data) = type_data.aliases.get(name) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Array(vec![MarkedString::LanguageString(
                            LanguageString {
                                language: "ic10".to_string(),
                                value: format!("alias {} {}", name, definition_data.value),
                            },
                        )]),
                        range: Some(Range::from(node.range()).into()),
                    }));
                }
                if let Some(definition_data) = type_data.labels.get(name) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Scalar(MarkedString::String(format!(
                            "Label on line {}",
                            definition_data.value + 1
                        ))),
                        range: Some(Range::from(node.range()).into()),
                    }));
                }
            }
            "operation" => {
                let Some(signature) = instructions::INSTRUCTIONS.get(name) else {
                    return Ok(None);
                };
                let mut content = name.to_string();
                for parameter in signature.0 {
                    content.push_str(&format!(" {parameter}"));
                }
                return Ok(Some(Hover {
                    contents: HoverContents::Array({
                        let mut v = Vec::new();
                        v.push(MarkedString::String(content));
                        if let Some(doc) = instructions::INSTRUCTION_DOCS.get(name) {
                            v.push(MarkedString::String(doc.to_string()));
                        }
                        v
                    }),
                    range: Some(Range::from(node.range()).into()),
                }));
            }
            "logictype" => {
                let Some(instruction_node) = node.find_parent("instruction") else {
                    return Ok(None);
                };

                let Some(operation_node) = instruction_node.child_by_field_name("operation") else {
                    return Ok(None);
                };

                let operation = operation_node
                    .utf8_text(document.content.as_bytes())
                    .unwrap();

                let (current_param, _) =
                    get_current_parameter(instruction_node, position.character as usize);

                let candidates = instructions::logictype_candidates(name);

                let types = if let Some(signature) = instructions::INSTRUCTIONS.get(operation) {
                    if let Some(param_type) = signature.0.get(current_param) {
                        param_type.intersection(&candidates)
                    } else {
                        candidates
                    }
                } else {
                    candidates
                };

                let strings = types
                    .iter()
                    .map(|typ| {
                        MarkedString::String(format!("# `{}` (`{}`)\n{}", name, typ, {
                            use instructions::DataType;
                            match typ {
                                DataType::LogicType => instructions::LOGIC_TYPE_DOCS.get(name),
                                DataType::SlotLogicType => instructions::SLOT_TYPE_DOCS.get(name),
                                DataType::BatchMode => instructions::BATCH_MODE_DOCS.get(name),
                                _ => None,
                            }
                            .unwrap_or(&"")
                        }))
                    })
                    .collect();

                return Ok(Some(Hover {
                    contents: HoverContents::Array(strings),
                    range: Some(Range::from(node.range()).into()),
                }));
            }
            _ => {}
        }
        Ok(None)
    }
}

impl Backend {
    fn node_at_position<'a>(&'a self, position: Position, tree: &'a Tree) -> Option<Node<'a>> {
        self.node_at_range(
            tower_lsp::lsp_types::Range::new(position.into(), position.into()).into(),
            tree,
        )
    }

    fn node_at_range<'a>(&'a self, range: Range, tree: &'a Tree) -> Option<Node<'a>> {
        let root = tree.root_node();
        let start = Position::from(range.0.start);
        let end = Position::from(range.0.end);
        let node = root.named_descendant_for_point_range(start.into(), end.into());

        node
    }

    async fn update_content(&self, uri: Url, mut text: String) {
        let mut files = self.files.write().await;

        if !text.ends_with("\n") {
            text.push('\n');
        }
        match files.entry(uri) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut parser = Parser::new();
                parser
                    .set_language(&tree_sitter_ic10::language())
                    .expect("Could not set language");
                let key = entry.key().clone();
                entry.insert(FileData {
                    document_data: DocumentData {
                        url: key,
                        tree: parser.parse(&text, None),
                        content: text,
                        parser,
                    },
                    type_data: TypeData::default(),
                });
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();
                entry.document_data.tree = entry.document_data.parser.parse(&text, None); // TODO
                entry.document_data.content = text;
            }
        }
    }

    async fn update_definitions(&self, uri: &Url, diagnostics: &mut Vec<Diagnostic>) {
        let mut files = self.files.write().await;
        let Some(file_data) = files.get_mut(uri) else {
            return;
        };
        let document = &file_data.document_data;
        let type_data = &mut file_data.type_data;

        if let Some(tree) = document.tree.as_ref() {
            type_data.defines.clear();
            type_data.aliases.clear();
            type_data.labels.clear();

            let mut cursor = QueryCursor::new();
            let query = Query::new(
                &tree_sitter_ic10::language(),
                "(instruction (operation \"define\"))@define
                         (instruction (operation \"alias\"))@alias
                         (instruction (operation \"label\"))@alias
                         (label (identifier)@label)",
            )
            .unwrap();

            let define_idx = query.capture_index_for_name("define").unwrap();
            let alias_idx = query.capture_index_for_name("alias").unwrap();
            let label_idx = query.capture_index_for_name("label").unwrap();

            let mut captures =
                cursor.captures(&query, tree.root_node(), document.content.as_bytes());

            while let Some((capture, _)) = captures.next() {
                let capture_idx = capture.captures[0].index;
                if capture_idx == define_idx || capture_idx == alias_idx {
                    if let Some(name_node) = capture.captures[0].node.child_by_field_name("operand")
                    {
                        let name = name_node.utf8_text(document.content.as_bytes()).unwrap();
                        let previous_range = {
                            if let Some(previous) = type_data.defines.get(name) {
                                Some(previous.range.clone())
                            } else if let Some(previous) = type_data.aliases.get(name) {
                                Some(previous.range.clone())
                            } else {
                                None
                            }
                        };
                        if let Some(previous_range) = previous_range {
                            diagnostics.push(Diagnostic::new(
                                Range::from(name_node.range()).into(),
                                Some(DiagnosticSeverity::ERROR),
                                None,
                                None,
                                "Duplicate definition".to_string(),
                                Some(vec![DiagnosticRelatedInformation {
                                    location: Location::new(
                                        document.url.clone(),
                                        previous_range.into(),
                                    ),
                                    message: "Previously defined here".to_string(),
                                }]),
                                None,
                            ));
                            continue;
                        } else {
                            let mut cursor = capture.captures[0].node.walk();
                            let value_node = capture.captures[0]
                                .node
                                .children_by_field_name("operand", &mut cursor)
                                .last();

                            if let Some(value_node) = value_node {
                                let value =
                                    value_node.utf8_text(document.content.as_bytes()).unwrap();
                                if capture.captures[0].index == define_idx {
                                    if value_node
                                        .child(0)
                                        .map(|x| x.kind())
                                        .map_or(false, |x| x != "number")
                                    {
                                        continue;
                                    }
                                    type_data.defines.insert(
                                        name.to_owned(),
                                        DefinitionData::new(
                                            name_node.range().into(),
                                            value.to_string(),
                                        ),
                                    );
                                } else if capture.captures[0].index == alias_idx {
                                    if value_node
                                        .child(0)
                                        .map(|x| x.kind())
                                        .map_or(false, |x| x != "register" && x != "device_spec")
                                    {
                                        continue;
                                    }
                                    type_data.aliases.insert(
                                        name.to_owned(),
                                        DefinitionData::new(
                                            name_node.range().into(),
                                            value.to_owned().into(),
                                        ),
                                    );
                                }
                            }
                        }
                    }
                } else if capture_idx == label_idx {
                    let name_node = capture.captures[0].node;
                    let name = name_node.utf8_text(document.content.as_bytes()).unwrap();
                    if let Some(previous) = type_data.get_range(name) {
                        diagnostics.push(Diagnostic::new(
                            Range::from(name_node.range()).into(),
                            Some(DiagnosticSeverity::ERROR),
                            None,
                            None,
                            "Duplicate definition".to_string(),
                            Some(vec![DiagnosticRelatedInformation {
                                location: Location::new(document.url.clone(), previous.into()),
                                message: "Previously defined here".to_string(),
                            }]),
                            None,
                        ));
                        continue;
                    }
                    type_data.labels.insert(
                        name.to_owned(),
                        DefinitionData {
                            range: name_node.range().into(),
                            value: name_node.start_position().row as u8,
                        },
                    );
                }
                //println!("{:#?}", capture);
            }
            // println!("{:#?}", type_data.defines);
            // println!("{:#?}", type_data.aliases);
            // println!("{:#?}", type_data.labels);
        }
    }

    async fn check_types(&self, uri: &Url, diagnostics: &mut Vec<Diagnostic>) {
        let files = self.files.read().await;
        let Some(file_data) = files.get(uri) else {
            return;
        };
        let document = &file_data.document_data;
        let type_data = &file_data.type_data;

        let Some(tree) = document.tree.as_ref() else {
            return;
        };

        let mut cursor = QueryCursor::new();
        let query = Query::new(&tree_sitter_ic10::language(), "(instruction)@a").unwrap();

        let mut captures = cursor.captures(&query, tree.root_node(), document.content.as_bytes());

        while let Some((capture, _)) = captures.next() {
            let capture = capture.captures[0].node;

            if let Some(operation_node) = capture.child_by_field_name("operation") {
                let operation = operation_node
                    .utf8_text(document.content.as_bytes())
                    .unwrap();
                let Some(signature) = instructions::INSTRUCTIONS.get(operation) else {
                    if operation != "define" && operation != "alias" && operation != "label" {
                        diagnostics.push(Diagnostic::new(
                            Range::from(operation_node.range()).into(),
                            Some(DiagnosticSeverity::INFORMATION),
                            None,
                            None,
                            format!("Unsupported instruction"),
                            None,
                            None,
                        ));
                    }
                    continue;
                };

                let mut argument_count = 0;
                let mut tree_cursor = capture.walk();
                let operands = capture.children_by_field_name("operand", &mut tree_cursor);
                let mut parameters = signature.0.iter();

                let mut first_superfluous_arg = None;

                for operand in operands {
                    use instructions::DataType;
                    argument_count = argument_count + 1;
                    let Some(parameter) = parameters.next() else {
                        if first_superfluous_arg.is_none() {
                            first_superfluous_arg = Some(operand);
                        }
                        continue;
                    };

                    let mut types = Vec::new();
                    let typ = match operand.named_child(0).unwrap().kind() {
                        "register" => instructions::Union(&[DataType::Register]),
                        "device_spec" => instructions::Union(&[DataType::Device]),
                        "number" => instructions::Union(&[DataType::Number]),
                        "logictype" => {
                            let ident = operand
                                .named_child(0)
                                .unwrap()
                                .utf8_text(document.content.as_bytes())
                                .unwrap();

                            if instructions::LOGIC_TYPES.contains(ident) {
                                types.push(DataType::LogicType);
                            }
                            if instructions::SLOT_LOGIC_TYPES.contains(ident) {
                                types.push(DataType::SlotLogicType);
                            }
                            if instructions::BATCH_MODES.contains(ident) {
                                types.push(DataType::BatchMode);
                            }
                            if instructions::REAGENT_MODES.contains(ident) {
                                types.push(DataType::ReagentMode);
                            }
                            instructions::Union(types.as_slice())
                        }
                        "identifier" => {
                            let ident = operand
                                .named_child(0)
                                .unwrap()
                                .utf8_text(document.content.as_bytes())
                                .unwrap();
                            if parameter.match_type(DataType::Name) {
                                instructions::Union(&[DataType::Name])
                            } else if type_data.defines.contains_key(ident)
                                || type_data.labels.contains_key(ident)
                            {
                                instructions::Union(&[DataType::Number])
                            } else if let Some(type_data) = type_data.aliases.get(ident) {
                                match type_data.value {
                                    AliasValue::Device(_) => {
                                        instructions::Union(&[DataType::Device])
                                    }
                                    AliasValue::Register(_) => {
                                        instructions::Union(&[DataType::Register])
                                    }
                                }
                            } else {
                                diagnostics.push(Diagnostic::new(
                                    Range::from(operand.range()).into(),
                                    Some(DiagnosticSeverity::ERROR),
                                    None,
                                    None,
                                    format!("Unknown identifier"),
                                    None,
                                    None,
                                ));
                                continue;
                            }
                        }
                        _ => {
                            // WTF
                            continue;
                        }
                    };

                    if !parameter.match_union(&typ) {
                        diagnostics.push(Diagnostic::new(
                            Range::from(operand.range()).into(),
                            Some(DiagnosticSeverity::ERROR),
                            None,
                            None,
                            format!("Type mismatch. Found {}, expected {}", typ, parameter),
                            None,
                            None,
                        ));
                    }
                }
                if argument_count > signature.0.len() {
                    let plural_str = if argument_count - signature.0.len() > 1 {
                        "s"
                    } else {
                        ""
                    };

                    diagnostics.push(Diagnostic::new(
                        tower_lsp::lsp_types::Range::new(
                            Position::from(first_superfluous_arg.unwrap().start_position()).into(),
                            Position::from(capture.end_position()).into(),
                        ),
                        Some(DiagnosticSeverity::ERROR),
                        None,
                        None,
                        format!(
                            "Superfluous argument{}. '{}' only requires {} arguments.",
                            plural_str,
                            operation,
                            signature.0.len()
                        ),
                        None,
                        None,
                    ));
                    continue;
                }
                if argument_count != signature.0.len() {
                    diagnostics.push(Diagnostic::new(
                        Range::from(capture.range()).into(),
                        Some(DiagnosticSeverity::ERROR),
                        None,
                        None,
                        "Invalid number of arguments".to_string(),
                        None,
                        None,
                    ));
                }
            }
        }
    }

    async fn run_diagnostics(&self, uri: &Url) {
        let mut diagnostics = Vec::new();

        // Collect definitions
        self.update_definitions(uri, &mut diagnostics).await;

        let config = self.config.read().await;
        let files = self.files.read().await;
        let Some(file_data) = files.get(uri) else {
            return;
        };

        let document = &file_data.document_data;
        let Some(tree) = document.tree.as_ref() else {
            return;
        };

        // Syntax errors
        {
            let mut cursor = QueryCursor::new();
            let query = Query::new(&tree_sitter_ic10::language(), "(ERROR)@error").unwrap();
            let mut captures =
                cursor.captures(&query, tree.root_node(), document.content.as_bytes());
            while let Some((capture, _)) = captures.next() {
                diagnostics.push(Diagnostic::new(
                    Range::from(capture.captures[0].node.range()).into(),
                    Some(DiagnosticSeverity::ERROR),
                    None,
                    None,
                    "Syntax error".to_string(),
                    None,
                    None,
                ));
            }
        }

        // Find invalid instructions
        {
            let mut cursor = QueryCursor::new();
            let query = Query::new(
                &tree_sitter_ic10::language(),
                "(instruction (invalid_instruction)@error)",
            )
            .unwrap();
            let mut captures =
                cursor.captures(&query, tree.root_node(), document.content.as_bytes());
            while let Some((capture, _)) = captures.next() {
                diagnostics.push(Diagnostic::new(
                    Range::from(capture.captures[0].node.range()).into(),
                    Some(DiagnosticSeverity::ERROR),
                    None,
                    None,
                    "Invalid instruction".to_string(),
                    None,
                    None,
                ));
            }
        }

        // Type check
        self.check_types(uri, &mut diagnostics).await;

        // Overlength checks
        {
            let mut cursor = QueryCursor::new();

            let query = Query::new(&tree_sitter_ic10::language(), "(instruction)@x").unwrap();
            let mut captures =
                cursor.captures(&query, tree.root_node(), document.content.as_bytes());
            while let Some((capture, _)) = captures.next() {
                let node = capture.captures[0].node;
                if node.end_position().column > config.max_columns {
                    diagnostics.push(Diagnostic {
                        range: LspRange::new(
                            LspPosition::new(
                                node.end_position().row as u32,
                                config.max_columns as u32,
                            ),
                            Position::from(node.end_position()).into(),
                        ),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: format!("Instruction past column {}", config.max_columns),
                        ..Default::default()
                    });
                }
            }

            if config.warn_overcolumn_comment {
                let query = Query::new(&tree_sitter_ic10::language(), "(comment)@x").unwrap();
                let mut captures =
                    cursor.captures(&query, tree.root_node(), document.content.as_bytes());
                while let Some((capture, _)) = captures.next() {
                    let node = capture.captures[0].node;
                    if node.end_position().column > config.max_columns {
                        diagnostics.push(Diagnostic {
                            range: LspRange::new(
                                LspPosition::new(
                                    node.end_position().row as u32,
                                    config.max_columns as u32,
                                ),
                                Position::from(node.end_position()).into(),
                            ),
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!("Comment past column {}", config.max_columns),
                            ..Default::default()
                        });
                    }
                }
            }

            cursor.set_point_range(
                tree_sitter::Point::new(config.max_lines, 0)
                    ..tree_sitter::Point::new(usize::MAX, usize::MAX),
            );
            let query = Query::new(&tree_sitter_ic10::language(), "(instruction)@x").unwrap();
            let mut captures =
                cursor.captures(&query, tree.root_node(), document.content.as_bytes());

            while let Some((capture, _)) = captures.next() {
                let node = capture.captures[0].node;
                diagnostics.push(Diagnostic {
                    range: Range::from(node.range()).into(),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Instruction past line {}", config.max_lines),
                    ..Default::default()
                });
            }

            if config.warn_overline_comment {
                let query = Query::new(&tree_sitter_ic10::language(), "(comment)@x").unwrap();
                let mut captures =
                    cursor.captures(&query, tree.root_node(), document.content.as_bytes());
                while let Some((capture, _)) = captures.next() {
                    let node = capture.captures[0].node;
                    diagnostics.push(Diagnostic {
                        range: Range::from(node.range()).into(),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!("Comment past line {}", config.max_lines),
                        ..Default::default()
                    });
                }
            }
        }

        // Absolute jump to number lint
        {
            const BRANCH_INSTRUCTIONS: phf::Set<&'static str> = phf_set!(
                "bdns", "bdnsal", "bdse", "bdseal", "bap", "bapz", "bapzal", "beq", "beqal",
                "beqz", "beqzal", "bge", "bgeal", "bgez", "bgezal", "bgt", "bgtal", "bgtz",
                "bgtzal", "ble", "bleal", "blez", "blezal", "blt", "bltal", "bltz", "bltzal",
                "bna", "bnaz", "bnazal", "bne", "bneal", "bnez", "bnezal", "j", "jal", "bdnvl",
                "bdnvs"
            );
            let mut cursor = QueryCursor::new();
            let query = Query::new(
                &tree_sitter_ic10::language(),
                "(instruction operand: (operand (number))) @x",
            )
            .unwrap();
            let mut tree_cursor = tree.walk();
            let mut captures =
                cursor.captures(&query, tree.root_node(), document.content.as_bytes());
            while let Some((capture, _)) = captures.next() {
                let capture = capture.captures[0].node;
                let Some(operation_node) = capture.child_by_field_name("operation") else {
                    continue;
                };
                let operation = operation_node
                    .utf8_text(document.content.as_bytes())
                    .unwrap();
                if !BRANCH_INSTRUCTIONS.contains(operation) {
                    continue;
                }

                tree_cursor.reset(capture);
                let Some(last_operand) = capture
                    .children_by_field_name("operand", &mut tree_cursor)
                    .into_iter()
                    .last()
                else {
                    continue;
                };
                let last_operand = last_operand.child(0).unwrap();

                if last_operand.kind() == "number" {
                    diagnostics.push(Diagnostic::new(
                        Range::from(capture.range()).into(),
                        Some(DiagnosticSeverity::WARNING),
                        Some(NumberOrString::String(LINT_ABSOLUTE_JUMP.to_string())),
                        None,
                        "Absolute jump to line number".to_string(),
                        None,
                        None,
                    ));
                }
            }
        }

        // Number batch mode
        {
            let mut cursor = QueryCursor::new();
            let query = Query::new(
                &tree_sitter_ic10::language(),
                "(instruction (operation)@op (operand (number)@n) .)",
            )
            .unwrap();

            let mut matches = cursor.matches(&query, tree.root_node(), document.content.as_bytes());

            while let Some(query_match) = matches.next() {
                {
                    let operation_node = query_match.captures[0].node;
                    let operation_text = operation_node
                        .utf8_text(document.content.as_bytes())
                        .unwrap();
                    if !operation_text.starts_with("lb") {
                        continue;
                    }
                }
                let node = query_match.captures[1].node;

                let Ok(value) = node
                    .utf8_text(document.content.as_bytes())
                    .unwrap()
                    .parse::<u8>()
                else {
                    diagnostics.push(Diagnostic {
                        range: Range::from(node.range()).into(),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "Use of non-integer batch mode".to_string(),
                        ..Default::default()
                    });
                    continue;
                };

                let Some(replacement) = instructions::BATCH_MODE_LOOKUP.get(&value) else {
                    diagnostics.push(Diagnostic {
                        range: Range::from(node.range()).into(),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "Invalid batch mode".to_string(),
                        ..Default::default()
                    });
                    continue;
                };

                diagnostics.push(Diagnostic {
                    range: Range::from(node.range()).into(),
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String(LINT_NUMBER_BATCH_MODE.to_string())),
                    message: "Use of literal number for batch mode".to_string(),
                    data: Some(Value::String(replacement.to_string())),
                    ..Default::default()
                });
            }
        }

        // Number reagent mode
        {
            let mut cursor = QueryCursor::new();
            let query = Query::new(
                &tree_sitter_ic10::language(),
                "(instruction (operation \"lr\") . (operand) . (operand) . (operand (number)@n))",
            )
            .unwrap();

            let mut captures =
                cursor.captures(&query, tree.root_node(), document.content.as_bytes());

            while let Some((capture, _)) = captures.next() {
                let node = capture.captures[0].node;

                let Ok(value) = node
                    .utf8_text(document.content.as_bytes())
                    .unwrap()
                    .parse::<u8>()
                else {
                    diagnostics.push(Diagnostic {
                        range: Range::from(node.range()).into(),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "Use of non-integer reagent mode".to_string(),
                        ..Default::default()
                    });
                    continue;
                };

                let Some(replacement) = instructions::REAGENT_MODE_LOOKUP.get(&value) else {
                    diagnostics.push(Diagnostic {
                        range: Range::from(node.range()).into(),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "Invalid reagent mode".to_string(),
                        ..Default::default()
                    });
                    continue;
                };

                diagnostics.push(Diagnostic {
                    range: Range::from(node.range()).into(),
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String(LINT_NUMBER_REAGENT_MODE.to_string())),
                    message: "Use of literal number for reagent mode".to_string(),
                    data: Some(Value::String(replacement.to_string())),
                    ..Default::default()
                });
            }
        }

        self.client
            .publish_diagnostics(uri.to_owned(), diagnostics, None)
            .await;
    }
}

fn get_current_parameter(instruction_node: Node, position: usize) -> (usize, Option<Node>) {
    let mut ret: usize = 0;
    let mut cursor = instruction_node.walk();
    for operand in instruction_node.children_by_field_name("operand", &mut cursor) {
        if operand.end_position().column > position {
            break;
        }
        ret += 1;
    }

    let operand = instruction_node
        .children_by_field_name("operand", &mut cursor)
        .nth(ret);

    cursor.reset(instruction_node);
    (ret, operand)
}

trait NodeEx: Sized {
    fn find_parent(&self, kind: &str) -> Option<Self>;
    fn query<'a>(&'a self, query: &str, content: impl AsRef<[u8]>) -> Option<Node<'a>>;
}

impl<'a> NodeEx for Node<'a> {
    fn find_parent(&self, kind: &str) -> Option<Self> {
        let mut cur = self.clone();
        while cur.kind() != kind {
            cur = cur.parent()?;
        }
        Some(cur)
    }

    fn query(&self, query: &str, content: impl AsRef<[u8]>) -> Option<Node<'a>> {
        let mut cursor = QueryCursor::new();
        let query = Query::new(&tree_sitter_ic10::language(), query).unwrap();

        let mut captures = cursor.captures(&query, self.clone(), content.as_ref());
        captures
            .next()
            .map(|x| x.0.captures)
            .and_then(|x| x.get(0))
            .map(|x| x.node)
    }
}

#[tokio::main]
async fn main() {
    use clap::Parser as _;
    let cli = cli::Cli::parse();

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_ic10::language())
        .expect("Failed to set language");

    let (service, socket) = LspService::new(|client| Backend {
        client,
        files: Arc::new(RwLock::new(HashMap::new())),
        config: Arc::new(RwLock::new(Configuration::default())),
    });

    if !cli.listen && cli.host.is_none() {
        // stdin/stdout
        Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
            .serve(service)
            .await;
    } else if cli.listen {
        // listen

        let host = cli
            .host
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed("127.0.0.1"))
            .parse::<Ipv4Addr>()
            .expect("Could not parse IP address");

        let port = cli.port.unwrap_or(9257);

        let stream = {
            let listener = TcpListener::bind((host, port)).await.unwrap();
            let (stream, _) = listener.accept().await.unwrap();
            stream
        };

        let (input, output) = tokio::io::split(stream);
        Server::new(input, output, socket).serve(service).await;
    } else {
        let host = cli.host.expect("No host given");
        let port = cli.port.expect("No port given");

        let stream = TcpStream::connect((host, port))
            .await
            .expect("Could not open TCP stream");

        let (input, output) = tokio::io::split(stream);
        Server::new(input, output, socket).serve(service).await;
    }
}

#[derive(Clone, Copy)]
struct Position(tower_lsp::lsp_types::Position);

#[derive(Clone, Debug)]
struct Range(tower_lsp::lsp_types::Range);

impl Range {
    pub fn contains(&self, position: Position) -> bool {
        let (start_line, start_char) = (self.0.start.line, self.0.start.character);
        let (end_line, end_char) = (self.0.end.line, self.0.end.character);
        let (line, character) = (position.0.line, position.0.character);

        (line > start_line && line < end_line)
            || (line == start_line && character >= start_char)
            || (line == end_line && character <= end_char)
    }
}

impl From<tree_sitter::Point> for Position {
    fn from(value: tree_sitter::Point) -> Self {
        Position(tower_lsp::lsp_types::Position::new(
            value.row as u32,
            value.column as u32,
        ))
    }
}

impl From<tower_lsp::lsp_types::Position> for Position {
    fn from(value: tower_lsp::lsp_types::Position) -> Self {
        Position(value)
    }
}

impl From<Position> for tower_lsp::lsp_types::Position {
    fn from(value: Position) -> Self {
        value.0
    }
}

impl From<Position> for tree_sitter::Point {
    fn from(value: Position) -> Self {
        tree_sitter::Point {
            row: value.0.line as usize,
            column: value.0.character as usize,
        }
    }
}

impl From<tree_sitter::Range> for Range {
    fn from(value: tree_sitter::Range) -> Self {
        Range(tower_lsp::lsp_types::Range::new(
            Position::from(value.start_point).into(),
            Position::from(value.end_point).into(),
        ))
    }
}

impl From<tower_lsp::lsp_types::Range> for Range {
    fn from(value: tower_lsp::lsp_types::Range) -> Self {
        Range(value)
    }
}

impl From<Range> for tower_lsp::lsp_types::Range {
    fn from(value: Range) -> Self {
        value.0
    }
}
