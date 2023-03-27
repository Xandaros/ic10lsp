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
        CompletionParams, CompletionResponse, Diagnostic, DiagnosticRelatedInformation,
        DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
        ExecuteCommandOptions, ExecuteCommandParams, GotoDefinitionParams, GotoDefinitionResponse,
        Hover, HoverContents, HoverParams, HoverProviderCapability, InitializeParams,
        InitializeResult, InitializedParams, LanguageString, Location, MarkedString, MessageType,
        NumberOrString, OneOf, ParameterInformation, ParameterLabel, PositionEncodingKind,
        ServerCapabilities, ServerInfo, SignatureHelp, SignatureHelpOptions, SignatureHelpParams,
        SignatureInformation, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Url,
        WorkDoneProgressOptions, WorkspaceEdit,
    },
    Client, LanguageServer, LspService, Server,
};
use tree_sitter::{Node, Parser, Query, QueryCursor, Tree};

mod cli;
mod instructions;

const LINT_ABSOLUTE_JUMP: &'static str = "L001";

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
    defines: HashMap<String, DefinitionData<f64>>,
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

struct Backend {
    client: Client,
    files: Arc<RwLock<HashMap<Url, FileData>>>,
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
                    commands: vec!["ic10.debug".to_string()],
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec![" ".to_string()]),
                    retrigger_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                position_encoding: utf8_supported.then_some(PositionEncodingKind::UTF8),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![" ".to_string()]),
                    completion_item: Some(CompletionOptionsCompletionItem {
                        label_details_support: Some(true),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "ic10lsp".to_string(),
                version: Some("1.0.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {}

    async fn execute_command(&self, _params: ExecuteCommandParams) -> Result<Option<Value>> {
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
                        ..Default::default()
                    });
                }
            }
            let length = completions.len();
            completions[start_entries..length].sort_by(|x, y| x.label.cmp(&y.label));
        }

        fn param_completions<T: std::fmt::Display>(
            map: &HashMap<String, DefinitionData<T>>,
            detail: &str,
            param_type: &instructions::Union,
            completions: &mut Vec<CompletionItem>,
        ) where
            DefinitionData<T>: HasType,
        {
            let start_entries = completions.len();
            for (identifier, value_data) in map.iter() {
                let value = &value_data.value;
                if param_type.match_type(value_data.get_type()) {
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
            let global_prefix = &text[..position.0.character as usize + 1];

            if global_prefix.chars().all(char::is_whitespace) {
                instruction_completions("", &mut ret);
            } else {
                let Some(line_node) = node.find_parent("line") else {
                    return Ok(None);
                };

                let Some(instruction_node) = line_node.query("(instruction)@x", file_data.document_data.content.as_bytes()) else {
                    return Ok(None);
                };

                let Some(operation_node) = instruction_node.child_by_field_name("operation") else {
                    return Ok(None);
                };

                let text = operation_node
                    .utf8_text(file_data.document_data.content.as_bytes())
                    .unwrap();

                let current_param = {
                    let mut ret: usize = 0;
                    let mut cursor = instruction_node.walk();
                    for operand in instruction_node.children_by_field_name("operand", &mut cursor) {
                        if operand.end_position().column as u32 > position.0.character {
                            break;
                        }
                        ret += 1;
                    }
                    ret
                };

                let Some(signature) = instructions::INSTRUCTIONS.get(text) else {
                    return Ok(None);
                };

                let Some(param_type) = signature.0.get(current_param) else {
                    return Ok(None);
                };

                if !text.starts_with("br") && text.starts_with("b") || text == "j" || text == "jal"
                {
                    param_completions(&file_data.type_data.labels, " label", param_type, &mut ret);

                    param_completions(
                        &file_data.type_data.defines,
                        " define",
                        param_type,
                        &mut ret,
                    );

                    param_completions(&file_data.type_data.aliases, " alias", param_type, &mut ret);
                } else {
                    param_completions(
                        &file_data.type_data.defines,
                        " define",
                        param_type,
                        &mut ret,
                    );

                    param_completions(&file_data.type_data.aliases, " alias", param_type, &mut ret);

                    param_completions(&file_data.type_data.labels, " label", param_type, &mut ret);
                }
            }
        }

        Ok(Some(CompletionResponse::Array(ret)))

        // Ok(Some(CompletionResponse::Array(vec![CompletionItem {
        //     label: "test".to_string(),
        //     // label_details: todo!(),
        //     // kind: todo!(),
        //     // detail: todo!(),
        //     // documentation: todo!(),
        //     // deprecated: todo!(),
        //     // preselect: todo!(),
        //     // sort_text: todo!(),
        //     // filter_text: todo!(),
        //     // insert_text: todo!(),
        //     // insert_text_format: todo!(),
        //     // insert_text_mode: todo!(),
        //     // text_edit: todo!(),
        //     // additional_text_edits: todo!(),
        //     // command: todo!(),
        //     // commit_characters: todo!(),
        //     // data: todo!(),
        //     // tags: todo!(),
        //     ..Default::default()
        // }])))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = Position::from(params.text_document_position_params.position);

        let files = self.files.read().await;
        let Some(file_data) = files.get(&uri) else {
            return Err(tower_lsp::jsonrpc::Error::invalid_request());
        };

        let Some(ref tree) = file_data.document_data.tree else {
            return Err(tower_lsp::jsonrpc::Error::internal_error());
        };

        let Some(node) = self.node_at_position(position, tree) else {
            return Ok(None);
        };

        let Some(line_node) = node.find_parent("line") else {
            return Ok(None);
        };

        let Some(instruction_node) = line_node.query("(instruction)@x", file_data.document_data.content.as_bytes()) else {
            return Ok(None);
        };

        let Some(operation_node) = instruction_node.child_by_field_name("operation") else {
            return Ok(None);
        };

        let text = operation_node
            .utf8_text(file_data.document_data.content.as_bytes())
            .unwrap();

        let current_param = {
            let mut ret = 0;
            let mut cursor = instruction_node.walk();
            for operand in instruction_node.children_by_field_name("operand", &mut cursor) {
                ret += 1;
                if operand.start_position().column as u32 > position.0.character {
                    break;
                }
            }
            ret
        };

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
                documentation: None,
                parameters: Some(
                    parameters
                        .iter()
                        .map(|offset| ParameterInformation {
                            label: ParameterLabel::LabelOffsets(offset.to_owned()),
                            documentation: None,
                        })
                        .collect(),
                ),
                active_parameter: Some(current_param),
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

        let Some(ref tree) = file_data.document_data.tree else {
            return Err(tower_lsp::jsonrpc::Error::invalid_request());
        };

        'diagnostics: for diagnostic in params.context.diagnostics {
            let Some(node) = self.node_at_range(params.range.into(), tree) else {
                        continue;
                    };

            let Some(line_node) = node.find_parent("line") else { continue 'diagnostics; };

            let Some(NumberOrString::String(code)) = diagnostic.code.clone() else {continue;};
            match code.as_str() {
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

                    if let Some(node) = line_node.query(
                        "(instruction (operation)@x)",
                        file_data.document_data.content.as_bytes(),
                    ) {
                        let text = node
                            .utf8_text(file_data.document_data.content.as_bytes())
                            .unwrap();

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
                                    file_data.document_data.url.clone(),
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
        let Some(file_data) = files.get(&params.text_document_position_params.text_document.uri) else {
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
        let Some(file_data) = files.get(&params.text_document_position_params.text_document.uri) else {
            return Err(tower_lsp::jsonrpc::Error::internal_error());
        };
        let document = &file_data.document_data;
        let type_data = &file_data.type_data;

        let position = params.text_document_position_params.position;

        if let Some(tree) = document.tree.as_ref() {
            let root = tree.root_node();
            let node = root.named_descendant_for_point_range(
                tree_sitter::Point::new(position.line as usize, position.character as usize),
                tree_sitter::Point::new(position.line as usize, position.character as usize + 1),
            );

            if let Some(node) = node {
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
                        let Some(signature) =  instructions::INSTRUCTIONS.get(name) else {
                            return Ok(None);
                        };
                        let mut content = name.to_string();
                        for parameter in signature.0 {
                            content.push_str(&format!(" {parameter}"));
                        }
                        return Ok(Some(Hover {
                            contents: HoverContents::Scalar(MarkedString::String(content)),
                            range: Some(Range::from(node.range()).into()),
                        }));
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }
}

impl Backend {
    fn node_at_position<'a>(&'a self, position: Position, tree: &'a Tree) -> Option<Node> {
        self.node_at_range(
            tower_lsp::lsp_types::Range::new(position.into(), position.into()).into(),
            tree,
        )
    }

    fn node_at_range<'a>(&'a self, range: Range, tree: &'a Tree) -> Option<Node> {
        let root = tree.root_node();
        let start = Position::from(range.0.start);
        let end = Position::from(range.0.end);
        let node = root.named_descendant_for_point_range(start.into(), end.into());

        node
    }

    async fn update_content(&self, uri: Url, text: String) {
        let mut files = self.files.write().await;
        match files.entry(uri) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut parser = Parser::new();
                parser
                    .set_language(tree_sitter_ic10::language())
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
                let mut entry = entry.get_mut();
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
                tree_sitter_ic10::language(),
                "(instruction (operation \"define\"))@define
                         (instruction (operation \"alias\"))@alias
                         (instruction (operation \"label\"))@alias
                         (label (identifier)@label)",
            )
            .unwrap();

            let define_idx = query.capture_index_for_name("define").unwrap();
            let alias_idx = query.capture_index_for_name("alias").unwrap();
            let label_idx = query.capture_index_for_name("label").unwrap();

            let captures = cursor.captures(&query, tree.root_node(), document.content.as_bytes());

            for (capture, _) in captures {
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
                                        diagnostics.push(Diagnostic::new(
                                            Range::from(value_node.range()).into(),
                                            Some(DiagnosticSeverity::ERROR),
                                            None,
                                            None,
                                            "Expected literal number for `define`".to_string(),
                                            None,
                                            None,
                                        ));
                                        continue;
                                    }
                                    type_data.defines.insert(
                                        name.to_owned(),
                                        DefinitionData::new(
                                            name_node.range().into(),
                                            value.parse().unwrap(),
                                        ),
                                    );
                                } else if capture.captures[0].index == alias_idx {
                                    if value_node
                                        .child(0)
                                        .map(|x| x.kind())
                                        .map_or(false, |x| x != "register" && x != "device")
                                    {
                                        diagnostics.push(Diagnostic::new(
                                            Range::from(value_node.range()).into(),
                                            Some(DiagnosticSeverity::ERROR),
                                            None,
                                            None,
                                            "Expected literal register or device for `alias`"
                                                .to_string(),
                                            None,
                                            None,
                                        ));
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
        let query = Query::new(tree_sitter_ic10::language(), "(instruction)@a").unwrap();

        let captures = cursor.captures(&query, tree.root_node(), document.content.as_bytes());

        for (capture, _) in captures {
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

                    let typ = match operand.named_child(0).unwrap().kind() {
                        "register" => DataType::Register,
                        "device" => DataType::Device,
                        "number" => DataType::Number,
                        "logictype" => {
                            let ident = operand
                                .named_child(0)
                                .unwrap()
                                .utf8_text(document.content.as_bytes())
                                .unwrap();
                            if instructions::LOGIC_TYPES.contains(ident)
                                && instructions::SLOT_LOGIC_TYPES.contains(ident)
                            {
                                DataType::EitherLogicType
                            } else if instructions::LOGIC_TYPES.contains(ident) {
                                DataType::LogicType
                            } else if instructions::SLOT_LOGIC_TYPES.contains(ident) {
                                DataType::SlotLogicType
                            } else {
                                // WTF
                                continue;
                            }
                        }
                        "identifier" => {
                            let ident = operand
                                .named_child(0)
                                .unwrap()
                                .utf8_text(document.content.as_bytes())
                                .unwrap();
                            if type_data.defines.contains_key(ident)
                                || type_data.labels.contains_key(ident)
                            {
                                DataType::Number
                            } else if let Some(type_data) = type_data.aliases.get(ident) {
                                match type_data.value {
                                    AliasValue::Device(_) => DataType::Device,
                                    AliasValue::Register(_) => DataType::Register,
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

                    if !parameter.match_type(typ) {
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

        let files = self.files.read().await;
        let Some(file_data) = files.get(uri) else {return;};

        // Find invalid instructions
        {
            let document = &file_data.document_data;
            let Some(tree) = document.tree.as_ref() else {
                return;
            };

            let mut cursor = QueryCursor::new();
            let query = Query::new(
                tree_sitter_ic10::language(),
                "(instruction (invalid_instruction)@error)",
            )
            .unwrap();
            let captures = cursor.captures(&query, tree.root_node(), document.content.as_bytes());
            for (capture, _) in captures {
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

        // Lints
        {
            let document = &file_data.document_data;
            let Some(tree) = document.tree.as_ref() else {
                return;
            };

            const BRANCH_INSTRUCTIONS: phf::Set<&'static str> = phf_set!(
                "bdns", "bdnsal", "bdse", "bdseal", "bap", "bapz", "bapzal", "beq", "beqal",
                "beqz", "beqzal", "bge", "bgeal", "bgez", "bgezal", "bgt", "bgtal", "bgtz",
                "bgtzal", "ble", "bleal", "blez", "blezal", "blt", "bltal", "bltz", "bltzal",
                "bna", "bnaz", "bnazal", "bne", "bneal", "bnez", "bnezal", "j", "jal"
            );
            let mut cursor = QueryCursor::new();
            let query = Query::new(
                tree_sitter_ic10::language(),
                "(instruction operand: (operand (number))) @x",
            )
            .unwrap();
            let mut tree_cursor = tree.walk();
            let captures = cursor.captures(&query, tree.root_node(), document.content.as_bytes());
            for (capture, _) in captures {
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
                let Some(last_operand) = capture.children_by_field_name("operand", &mut tree_cursor).into_iter().last() else {
                    continue;
                };
                let last_operand = last_operand.child(0).unwrap();

                if last_operand.kind() == "number" {
                    diagnostics.push(Diagnostic::new(
                        Range::from(capture.range()).into(),
                        Some(DiagnosticSeverity::WARNING),
                        Some(NumberOrString::String(LINT_ABSOLUTE_JUMP.to_owned())),
                        None,
                        "Absolute jump to line number".to_string(),
                        None,
                        None,
                    ));
                }
            }
        }

        self.client
            .publish_diagnostics(uri.to_owned(), diagnostics, None)
            .await;
    }
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
        let query = Query::new(tree_sitter_ic10::language(), query).unwrap();

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
        .set_language(tree_sitter_ic10::language())
        .expect("Failed to set language");

    let (service, socket) = LspService::new(|client| Backend {
        client,
        files: Arc::new(RwLock::new(HashMap::new())),
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