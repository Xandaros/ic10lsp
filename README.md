# IC10LSP

A simple language server for the IC10 MIPS-like language in the game Stationeers.

Features:

- Completions (not fully)
- Hover information
- Signature help
- Goto definition
- Diagnostic information

![Demo](demo.gif)

## Configuration

The language server exposes the following configuration options:

| Key                         | Description                                      | Default |
| --------------------------- | ------------------------------------------------ | ------- |
| max_lines                   | Maximum number of lines                          | 128     |
| max_columns                 | Maximum number of columns                        | 52      |
| warnings.overline_comment   | Emit a warning on comments past the line limit   | true    |
| warnings.overcolumn_comment | Emit a warning on comments past the column limit | true    |

## Commands

The language server exposes the following commands:

| Command | Description                                            |
| ------- | ------------------------------------------------------ |
| version | Show a message with the version of the language server |
