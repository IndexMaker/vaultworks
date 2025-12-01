use clap::Parser;
use eyre::OptionExt;
use itertools::Itertools;
use regex::Regex;
use std::error::Error;
use std::fmt::{self, Display, Formatter, Write};
use std::{fs, usize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'i', long)]
    input_file: String,

    #[arg(short = 'o', long)]
    output_file: String,

    #[arg(short = 't', long)]
    output_type: String,
}

#[derive(Debug)]
struct VectorInstruction {
    mnemonic: String,
    example: String,
    operands: Vec<String>,
    description: Vec<String>,
}

#[derive(Debug)]
enum ParserError {
    Io(std::io::Error),
    RegexCreation(regex::Error),
    FileNotFound(String),
    FormatError(fmt::Error),
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParserError::Io(e) => write!(f, "I/O Error: {}", e),
            ParserError::RegexCreation(e) => write!(f, "Regex Creation Error: {}", e),
            ParserError::FileNotFound(name) => write!(f, "File Not Found: '{}'", name),
            ParserError::FormatError(e) => write!(f, "Formatting Error: {}", e),
        }
    }
}

impl Error for ParserError {}

impl From<std::io::Error> for ParserError {
    fn from(err: std::io::Error) -> Self {
        ParserError::Io(err)
    }
}

impl From<regex::Error> for ParserError {
    fn from(err: regex::Error) -> Self {
        ParserError::RegexCreation(err)
    }
}

impl From<fmt::Error> for ParserError {
    fn from(err: fmt::Error) -> Self {
        ParserError::FormatError(err)
    }
}

/// Parses the content of the Rust source file to extract components into VectorInstruction objects.
fn parse_instruction_set(file_content: &str) -> Result<Vec<VectorInstruction>, ParserError> {
    // Regex to capture:
    // 1. Mnemonic (Group 1: OP_[A-Z]+)
    // 2. Everything after the '//' comment delimiter (Group 2: .*)
    let line_pattern = Regex::new(r"pub\s+const\s+(OP_[A-Z]+):\s+u8\s+=\s+\d+;.*?//\s*(.*)")?;

    let mut parsed_instructions: Vec<VectorInstruction> = Vec::new();

    for line in file_content.lines() {
        if let Some(captures) = line_pattern.captures(line.trim()) {
            let full_mnemonic = captures.get(1).map_or("", |m| m.as_str());
            let comment_content = captures.get(2).map_or("", |m| m.as_str()).trim();

            // --- 1. Isolate Mnemonic and remove OP_ prefix ---
            let mnemonic = full_mnemonic.trim_start_matches("OP_").to_string();

            // Find the index of the first semicolon to separate Example/Operands from Descriptions
            if let Some(first_semicolon_index) = comment_content.find(';') {
                // --- 2. Isolate the Example (The part before the first ';') ---
                let example_with_mnemonic = &comment_content[..first_semicolon_index].trim();

                // Remove the mnemonic from the start of the example string
                let example_suffix = example_with_mnemonic.trim_start_matches(&mnemonic).trim();

                // Reconstruct the full example string (e.g., "LDV <vector_id>")
                let example = if example_suffix.is_empty() {
                    mnemonic.clone()
                } else {
                    format!("{} {}", mnemonic, example_suffix)
                };

                // --- Operands: Parse the space-separated parts of the example suffix ---
                let operands: Vec<String> = example_suffix
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();

                // --- 3. Isolate description splitting into separate string by ';' character ---
                let description_parts_raw = &comment_content[first_semicolon_index + 1..];

                let description: Vec<String> = description_parts_raw
                    .split(';')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                parsed_instructions.push(VectorInstruction {
                    mnemonic,
                    example,
                    operands,
                    description,
                });
            } else {
                // Handle case where line matches pattern but has no semicolon (only mnemonic/example)
                let example = comment_content.trim().to_string();
                parsed_instructions.push(VectorInstruction {
                    mnemonic,
                    example,
                    operands: vec![],
                    description: vec![],
                });
            }
        }
    }

    Ok(parsed_instructions)
}

pub fn luatex_sanitize_string(s: &str) -> String {
    // 1. Convert brackets [x] to $[x]$ for LaTeX math mode.
    // We replace the entire construct `[content]` with `$[content]$`.
    // The `expect` ensures the regex is valid; in a real app, this should be handled gracefully.
    let bracket_re = Regex::new(r"\[(.*?)\]").expect("Failed to create bracket regex");

    // Perform the bracket replacement first. The result is a new, owned String.
    // The pattern r"$$\[$1\]$$" substitutes the content of the brackets,
    // wrapping it in LaTeX math delimiters.
    let mut result = bracket_re.replace_all(s, r"$$[$1]$$").to_string();

    // 2. Handle remaining LaTeX control characters on the string after bracket replacement.
    // Note: We intentionally omit escaping '$' here, as they are now correctly used as math delimiters.

    result = result.replace('_', r"\_"); // Subscript
    result = result.replace('%', r"\%"); // Comment
    result = result.replace('&', r"\&"); // Alignment
    result = result.replace('#', r"\#"); // Parameter macro
    result = result.replace('{', r"\{"); // Grouping
    result = result.replace('}', r"\}"); // Grouping
    result = result.replace('~', r"\textasciitilde{}"); // Tilde
    result = result.replace('^', r"\textasciicircum{}"); // Superscript

    result
}

/// Formats the parsed instruction data into a readable tabular string.
fn format_as_ascii_table(instructions: &[VectorInstruction]) -> eyre::Result<String> {
    let mut output = String::new();

    if instructions.is_empty() {
        writeln!(
            &mut output,
            "No instructions were parsed from the input file."
        )?;
        return Ok(output);
    }

    // Determine maximum widths for alignment
    let mnemonic_w = instructions
        .iter()
        .map(|i| i.mnemonic.len())
        .max()
        .unwrap_or(10);
    let example_w = instructions
        .iter()
        .map(|i| i.example.len())
        .max()
        .unwrap_or(20);
    let operands_w = instructions
        .iter()
        .map(|i| i.operands.join(", ").len())
        .max()
        .unwrap_or(20);

    // Headers
    let header = format!(
        "{:width_m$} | {:width_e$} | {:width_o$} | DESCRIPTION",
        "MNEMONIC",
        "EXAMPLE",
        "OPERANDS",
        width_m = mnemonic_w,
        width_e = example_w,
        width_o = operands_w
    );

    // Calculate separator length dynamically
    let sep_len = header.len() + 10; // +10 for buffer if description is long
    let separator = "-".repeat(sep_len);

    writeln!(
        &mut output,
        "--- Vector Instruction Set (VIS) Parsing Results ---"
    )?;
    writeln!(&mut output, "{}", header)?;
    writeln!(&mut output, "{}", separator)?;

    // Alignment padding for subsequent description lines (under DESCRIPTION column)
    let alignment_padding = " ".repeat(mnemonic_w + example_w + operands_w + 6);

    for instr in instructions {
        let mnemonic = &instr.mnemonic;
        let example = &instr.example;
        let operands_str = instr.operands.join(", ");
        let descriptions = &instr.description;

        // Print the first line of description alongside the instruction details
        if let Some(first_desc) = descriptions.first() {
            writeln!(
                &mut output,
                "{:width_m$} | {:width_e$} | {:width_o$} | {}",
                mnemonic,
                example,
                operands_str,
                first_desc,
                width_m = mnemonic_w,
                width_e = example_w,
                width_o = operands_w
            )?;

            // Print subsequent description parts, aligned under the DESCRIPTION column
            for desc in descriptions.iter().skip(1) {
                writeln!(&mut output, "{} | {}", alignment_padding, desc)?;
            }
        } else {
            // Case with no detailed description parts
            writeln!(
                &mut output,
                "{:width_m$} | {:width_e$} | {:width_o$} | (No detailed description)",
                mnemonic,
                example,
                operands_str,
                width_m = mnemonic_w,
                width_e = example_w,
                width_o = operands_w
            )?;
        }
    }

    writeln!(
        &mut output,
        "\nTotal Instructions Parsed: {}",
        instructions.len()
    )?;

    Ok(output)
}

fn format_as_tex_longtable(instructions: &[VectorInstruction]) -> eyre::Result<String> {
    let mut output = String::new();

    writeln!(
        &mut output,
        "{}",
        r"\begin{longtable}{p{0.15\linewidth} p{0.75\linewidth}}
\caption{VIL Instruction Set Summary}\label{tab:vil_summary}\\
\toprule
\textbf{Mnemonic} & \textbf{Brief Description} \\
\midrule
\endfirsthead
\multicolumn{2}{c}%
{\tablename\ \thetable\ -- Continued from previous page} \\
\toprule
\textbf{Mnemonic} & \textbf{Brief Description} \\
\midrule
\endhead
\bottomrule
\endfoot
\bottomrule
\endlastfoot"
    )?;

    for instruction in instructions {
        writeln!(
            &mut output,
            r"\hyperlink{{inst:{}}}{{\texttt{{{}}}}} & {}. \\",
            instruction.mnemonic.to_ascii_lowercase(),
            instruction.mnemonic.to_ascii_uppercase(),
            luatex_sanitize_string(
                instruction
                    .description
                    .last()
                    .map(|s| s.as_str())
                    .unwrap_or("")
            )
        )?;
    }

    writeln!(&mut output, "{}", r"\end{longtable}")?;

    Ok(output)
}

fn format_as_tex_subsections(instructions: &[VectorInstruction]) -> eyre::Result<String> {
    let mut output = String::new();

    for instruction in instructions {
        writeln!(
            &mut output,
            r"\subsubsection{{\texttt{{{}}} Instruction\label{{inst:{}}}}}
",
            instruction.mnemonic.to_uppercase(),
            instruction.mnemonic.to_ascii_lowercase()
        )?;

        let (stack_args, result, description) = instruction
            .description
            .clone()
            .into_iter()
            .collect_tuple()
            .unwrap_or_else(|| {
                (
                    String::default(),
                    String::default(),
                    instruction
                        .description
                        .last()
                        .cloned()
                        .unwrap_or(String::default()),
                )
            });

        writeln!(
            &mut output,
            "{}",
            luatex_sanitize_string(description.as_str())
        )?;

        writeln!(&mut output, "\n{}", r"\noindent \textbf{Operands}")?;

        writeln!(&mut output, "\n{}", r"\begin{itemize}")?;
        if !instruction.operands.is_empty() {
            for arg in &instruction.operands {
                writeln!(
                    &mut output,
                    r"\item \texttt{{{}}}: \texttt{{u128}} Stack pos",
                    luatex_sanitize_string(arg)
                )?;
            }
        } else {
            writeln!(&mut output, "{}", r"\item (no operands)")?;
        }
        writeln!(&mut output, "{}", r"\end{itemize}")?;

        writeln!(&mut output, "\n{}", r"\noindent \textbf{Stack Args}")?;

        writeln!(&mut output, "\n{}", r"\begin{itemize}")?;

        writeln!(
            &mut output,
            r"\item \texttt{{{}}}: Input",
            luatex_sanitize_string(stack_args.as_str())
        )?;

        writeln!(&mut output, "{}", r"\end{itemize}")?;

        writeln!(&mut output, "\n{}", r"\noindent \textbf{Return}")?;

        writeln!(&mut output, "\n{}", r"\begin{itemize}")?;

        writeln!(
            &mut output,
            r"\item \texttt{{{}}}: Output",
            luatex_sanitize_string(result.as_str())
        )?;

        writeln!(&mut output, "{}", r"\end{itemize}")?;

        writeln!(
            &mut output,
            "\n{}",
            r"\noindent \textbf{Usage Example}

\begin{verbatim}
"
        )?;

        writeln!(&mut output, "    {}", instruction.example)?;
        writeln!(&mut output, "\n{}\n", r"\end{verbatim}")?;
    }

    Ok(output)
}

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    let input_path = cli.input_file;
    let output_path = cli.output_file;

    let rust_file_content = fs::read_to_string(&input_path)
        .map_err(|e| ParserError::FileNotFound(format!("{}: {}", &input_path, e)))?;

    let parsed_data = parse_instruction_set(&rust_file_content)?;

    let formatted_output = match cli.output_type.as_str() {
        "ascii-table" => format_as_ascii_table(&parsed_data)?,
        "tex-longtable" => format_as_tex_longtable(&parsed_data)?,
        "tex-subsections" => format_as_tex_subsections(&parsed_data)?,
        t => Err(eyre::eyre!("Invalid type {}", t))?,
    };

    fs::write(&output_path, formatted_output).map_err(|e| ParserError::Io(e))?;

    eprintln!(
        "Successfully parsed {} instructions and wrote to {}",
        parsed_data.len(),
        output_path
    );

    Ok(())
}
