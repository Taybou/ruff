//! We want to ensure that once formatted content stays the same when formatted again. This is known
//! as formatter stability or formatter idempotency. We already check this for our test cases, but
//! we also want to check it for entire repositories similar to what black does
#![allow(clippy::print_stdout)]

use anyhow::Context;
use clap;
use clap::Parser;
use log::debug;
use ruff::resolver::python_files_in_path;
use ruff_cli::args::CheckArgs;
use ruff_cli::resolve::resolve;
use ruff_python_formatter::format_module;
use similar::{ChangeTag, TextDiff};
use std::io::Write;
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{fs, io, iter};

#[derive(Copy, Clone, PartialEq, Eq, clap::ValueEnum, Default)]
pub(crate) enum Format {
    // Filenames only
    Minimal,
    // Filenames and reduced diff
    #[default]
    Default,
    // Full diff and invalid code
    Full,
}

#[derive(clap::Args)]
pub(crate) struct Args {
    /// Like `ruff check`'s files
    pub(crate) files: Vec<PathBuf>,
    #[arg(long, default_value_t, value_enum)]
    pub(crate) format: Format,
}

/// Generate ourself a try_parse_from impl for CheckArgs. This is a strange way to use clap but we
/// want the same behaviour as ruff_cli and clap seems to lack a way to parse directly to `Args`
/// instead of a `Parser`
#[derive(Debug, clap::Parser)]
struct WrapperArgs {
    #[clap(flatten)]
    check_args: CheckArgs,
}

pub(crate) fn main(args: &Args) -> anyhow::Result<()> {
    let start = Instant::now();

    // Find files to check (or in this case, format twice). Adapted from ruff_cli
    // First argument is ignored
    let dummy = PathBuf::from("check");
    let check_args_input = iter::once(&dummy).chain(&args.files);
    let check_args = WrapperArgs::try_parse_from(check_args_input)?.check_args;
    let (cli, overrides) = check_args.partition();
    let pyproject_config = resolve(
        cli.isolated,
        cli.config.as_deref(),
        &overrides,
        cli.stdin_filename.as_deref(),
    )?;
    let (paths, _resolver) = python_files_in_path(&cli.files, &pyproject_config, &overrides)?;
    assert!(!paths.is_empty(), "no python files in {:?}", cli.files);

    let errors = paths
        .into_iter()
        .map(|dir_entry| {
            // Doesn't make sense to recover here in this test script
            let file = dir_entry
                .expect("Iterating the files in the repository failed")
                .into_path();
            let result = match catch_unwind(|| check_file(&file)) {
                Ok(result) => result,
                Err(panic) => {
                    if let Ok(message) = panic.downcast::<String>() {
                        Err(FormatterStabilityError::Panic { message: *message })
                    } else {
                        Err(FormatterStabilityError::Panic {
                            // This should not happen, but it can
                            message: "(Panic didn't set a string message)".to_string(),
                        })
                    }
                }
            };
            (result, file)
        })
        // We only care about the errors
        .filter_map(|(result, file)| match result {
            Err(err) => Some((err, file)),
            Ok(()) => None,
        });

    // Don't collect the iterator so we already see errors while it's still processing
    for (error, file) in errors {
        match error {
            FormatterStabilityError::Unstable {
                formatted,
                reformatted,
            } => {
                println!("Unstable formatting {}", file.display());
                match args.format {
                    Format::Minimal => {}
                    Format::Default => {
                        diff_show_only_changes(
                            io::stdout().lock().by_ref(),
                            &formatted,
                            &reformatted,
                        )?;
                    }
                    Format::Full => {
                        let diff = TextDiff::from_lines(&formatted, &reformatted)
                            .unified_diff()
                            .header("Formatted once", "Formatted twice")
                            .to_string();
                        println!(
                            r#"Reformatting the formatted code a second time resulted in formatting changes.
---
{diff}---

Formatted once:
---
{formatted}---

Formatted twice:
---
{reformatted}---"#,
                        );
                    }
                }
            }
            FormatterStabilityError::InvalidSyntax { err, formatted } => {
                println!(
                    "Formatter generated invalid syntax {}: {}",
                    file.display(),
                    err
                );
                if args.format == Format::Full {
                    println!("---\n{}\n---\n", formatted);
                }
            }
            FormatterStabilityError::Panic { message } => {
                println!("Panic {}: {}", file.display(), message);
            }
            FormatterStabilityError::Other(err) => {
                println!("Uncategorized error {}: {}", file.display(), err);
            }
        }
    }
    let duration = start.elapsed();
    println!(
        "Formatting {} files twice took {:.2}s",
        cli.files.len(),
        duration.as_secs_f32()
    );

    Ok(())
}

/// A diff that only shows a header and changes, but nothing unchanged. This makes viewing multiple
/// errors easier.
fn diff_show_only_changes(
    writer: &mut impl Write,
    formatted: &str,
    reformatted: &str,
) -> io::Result<()> {
    for changes in TextDiff::from_lines(formatted, reformatted)
        .unified_diff()
        .iter_hunks()
    {
        for (idx, change) in changes
            .iter_changes()
            .filter(|change| change.tag() != ChangeTag::Equal)
            .enumerate()
        {
            if idx == 0 {
                writeln!(writer, "{}", changes.header())?;
            }
            write!(writer, "{}", change.tag())?;
            writer.write_all(change.value().as_bytes())?;
        }
    }
    Ok(())
}

#[derive(Debug)]
enum FormatterStabilityError {
    /// The error we're looking for: First and second pass are different
    Unstable {
        formatted: String,
        reformatted: String,
    },
    /// "Expected formatted code to be valid syntax: {}:\n---\n{}---\n",
    InvalidSyntax {
        err: anyhow::Error,
        // TODO(konstin): Reactivate this when the errors are fixed. Currently this would be too
        // verbose
        #[allow(dead_code)]
        formatted: String,
    },
    /// From `catch_unwind`
    Panic {
        message: String,
    },
    Other(anyhow::Error),
}

impl From<anyhow::Error> for FormatterStabilityError {
    fn from(error: anyhow::Error) -> Self {
        Self::Other(error)
    }
}

/// Runs the formatter twice on the given file. It does not write back to the file
fn check_file(input_path: &Path) -> Result<(), FormatterStabilityError> {
    let content = fs::read_to_string(input_path).context("Failed to read file")?;
    let printed = match format_module(&content) {
        Ok(printed) => printed,
        Err(err) => {
            return if err
                .to_string()
                .starts_with("Source contains syntax errors ")
            {
                debug!(
                    "Skipping {} with invalid first pass {}",
                    input_path.display(),
                    err
                );
                Ok(())
            } else {
                Err(err.into())
            };
        }
    };
    let formatted = printed.as_code();

    let reformatted = match format_module(formatted) {
        Ok(reformatted) => reformatted,
        Err(err) => {
            return Err(FormatterStabilityError::InvalidSyntax {
                err,
                formatted: formatted.to_string(),
            });
        }
    };

    if reformatted.as_code() != formatted {
        return Err(FormatterStabilityError::Unstable {
            formatted: formatted.to_string(),
            reformatted: reformatted.into_code(),
        });
    }
    Ok(())
}
