use crate::digest_ext::HashExt;
use clap::{CommandFactory, Parser};
use colored::Colorize;
use std::io::{BufRead, Read};
use strum::IntoEnumIterator;
mod digest_ext;
mod escape;
mod utils;

#[derive(
    Debug, Copy, Clone, clap::ValueEnum, strum::Display, strum::EnumString, strum::EnumIter,
)]
enum Algorithm {
    #[clap(name = "md5")]
    MD5,
    #[clap(name = "sha1")]
    SHA1,
    #[clap(name = "sha224")]
    SHA224,
    #[clap(name = "sha256", alias = "sha2")]
    SHA256,
    #[clap(name = "sha384")]
    SHA384,
    #[clap(name = "sha512")]
    SHA512,
    #[clap(name = "xxh32")]
    Xxh32,
    #[clap(name = "xxh64")]
    Xxh64,
    #[clap(name = "xxh3")]
    Xxh3,
}

#[derive(Debug, Parser)]
#[clap(version, about)]
struct Options {
    /// print shell completion script.
    #[clap(long)]
    completion: Option<clap_complete::Shell>,

    /// read in binary mode. (noop)
    #[clap(short, long, default_value = "false")]
    binary: bool,

    /// read in text mode. (noop)
    #[clap(short, long, default_value = "true")]
    text: bool,

    /// check for differences between the new and original file.
    #[clap(short, long, default_value = "false")]
    check: bool,

    /// create a BSD-style checksum.
    #[clap(long, default_value = "false", help_heading = "Display options")]
    tag: bool,

    /// end each output line with a NULL character instead of newline, and disable file name
    /// escaping.
    #[clap(short, long, default_value = "false", help_heading = "Display options")]
    zero: bool,

    /// don't fail or report status for missing files.
    #[clap(long, default_value = "false", help_heading = "Check mode options")]
    ignore_missing: bool,

    /// don't put OK for each successfully verified file.
    #[clap(long, default_value = "false", help_heading = "Check mode options")]
    quiet: bool,

    /// don't output anything. you can use status code to check for success.
    #[clap(long, default_value = "false", help_heading = "Check mode options")]
    status: bool,

    /// exit non-zero for improperly formatted checksum lines.
    #[clap(long, default_value = "false", help_heading = "Check mode options")]
    strict: bool,

    /// warn about improperly formatted checksum lines.
    #[clap(
        short,
        long,
        default_value = "false",
        help_heading = "Check mode options"
    )]
    warn: bool,

    /// use the specified algorithm to generate the checksum.
    #[clap(short, long, default_value = "sha256")]
    algorithm: Algorithm,

    /// buffer size for reading files, in bytes.
    #[clap(short = 'B', long, default_value = "8192", env = "CCSUM_BUFFER_SIZE")]
    buffer_size: usize,

    /// group output by last N segments of the path.
    #[clap(
        short,
        long,
        help_heading = "Group mode options",
        num_args = 0..=1,
        default_missing_value = "1",
        require_equals = true,
        value_parser = clap::value_parser!(u64).range(1..),
    )]
    group: Option<u64>,

    /// group output by last N segments of the path, and fail if any checksums in the group are
    /// different.
    #[clap(
        short = 'G',
        long,
        help_heading = "Group mode options",
        conflicts_with = "group",
        num_args = 0..=1,
        default_missing_value = "1",
        require_equals = true,
        value_parser = clap::value_parser!(u64).range(1..),
    )]
    group_with_check: Option<u64>,

    /// colorize the output, even if stdout is not a tty.
    #[clap(
        alias = "C",
        long,
        default_value = "false",
        help_heading = "Display options"
    )]
    color: bool,

    /// disable colorized output.
    #[clap(long, default_value = "false", help_heading = "Display options")]
    no_color: bool,

    /// the files to generate the checksum for.
    files: Vec<String>,
}

fn checksum_read(data: impl Read, algorithm: Algorithm, buffer_size: usize) -> Vec<u8> {
    match algorithm {
        Algorithm::MD5 => md5::Md5::hash(data, buffer_size),
        Algorithm::SHA1 => sha1::Sha1::hash(data, buffer_size),
        Algorithm::SHA224 => sha2::Sha224::hash(data, buffer_size),
        Algorithm::SHA256 => sha2::Sha256::hash(data, buffer_size),
        Algorithm::SHA384 => sha2::Sha384::hash(data, buffer_size),
        Algorithm::SHA512 => sha2::Sha512::hash(data, buffer_size),
        Algorithm::Xxh32 => xxhash_rust::xxh32::Xxh32::hash(data, buffer_size),
        Algorithm::Xxh64 => xxhash_rust::xxh64::Xxh64::hash(data, buffer_size),
        Algorithm::Xxh3 => xxhash_rust::xxh3::Xxh3::hash(data, buffer_size),
    }
}

fn checksum_file(file: &str, algorithm: Algorithm, buffer_size: usize) -> anyhow::Result<Vec<u8>> {
    let file = fs_err::File::open(file)?;
    Ok(checksum_read(&file, algorithm, buffer_size))
}

fn checksum_stdin(algorithm: Algorithm, buffer_size: usize) -> anyhow::Result<Vec<u8>> {
    Ok(checksum_read(std::io::stdin(), algorithm, buffer_size))
}

fn main() -> anyhow::Result<()> {
    let mut options = Options::parse();

    if let Some(shell) = options.completion {
        clap_complete::generate(
            shell,
            &mut Options::command(),
            "ccsum",
            &mut std::io::stdout(),
        );
        return Ok(());
    }

    if options.no_color {
        colored::control::set_override(false);
    } else if options.color {
        colored::control::set_override(true);
    }

    if options.files.is_empty() {
        options.files.push("-".to_string());
    }

    if options.check {
        do_check(&options)?;
    } else if options.group.is_some() || options.group_with_check.is_some() {
        do_checksum_with_group(&options)?;
    } else {
        do_checksum(&options)?;
    }

    Ok(())
}

fn do_checksum(options: &Options) -> anyhow::Result<()> {
    let mut anything_failed = false;
    for file in &options.files {
        let checksum = if file == "-" {
            checksum_stdin(options.algorithm, options.buffer_size)
        } else {
            checksum_file(file, options.algorithm, options.buffer_size)
        };
        let checksum = match checksum {
            Ok(checksum) => checksum,
            Err(e) => {
                eprintln!("{}: {}", file, e.to_string().red());
                anything_failed = true;
                continue;
            }
        };

        let (red, green, blue) = utils::checksum_to_color(&checksum, false);
        let checksum_hex = hex::encode(checksum);
        let colored_checksum = checksum_hex.color(colored::Color::TrueColor {
            r: red,
            g: green,
            b: blue,
        });

        let file_display = if options.zero {
            file.clone()
        } else {
            escape::escape(file)
        };
        let line = if options.tag {
            format!(
                "{} ({}) = {}",
                options.algorithm, file_display, colored_checksum
            )
        } else {
            format!("{}  {}", colored_checksum, file_display)
        };
        if options.zero {
            print!("{}\0", line);
        } else {
            println!("{}", line);
        }
    }

    if anything_failed {
        std::process::exit(1);
    }

    Ok(())
}

fn do_checksum_with_group(options: &Options) -> anyhow::Result<()> {
    let mut anything_failed = false;
    let mut anything_group_failed = false;
    let mut anything_succeeded = false;
    let n = options.group.or(options.group_with_check).unwrap() as usize;

    let mut groups = std::collections::HashMap::new();
    for file in &options.files {
        let (_head, tail) = utils::split_at_last_segments(file, n);
        groups.entry(tail).or_insert_with(Vec::new).push(file);
    }
    let mut groups = groups.into_iter().collect::<Vec<_>>();
    groups.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (_tail, files) in groups {
        let checksums = files
            .iter()
            .map(|&file| {
                let checksum = if file == "-" {
                    checksum_stdin(options.algorithm, options.buffer_size)
                } else {
                    checksum_file(file, options.algorithm, options.buffer_size)
                };
                let checksum = match checksum {
                    Ok(checksum) => checksum,
                    Err(e) => {
                        eprintln!("{}: {}", file, e.to_string().red());
                        anything_failed = true;
                        return None;
                    }
                };
                Some(checksum)
            })
            .collect::<Vec<_>>();

        let is_same = checksums.windows(2).all(|pair| pair[0] == pair[1]);

        for (checksum, file) in checksums.iter().zip(files) {
            let Some(checksum) = checksum else {
                continue;
            };
            let (red, green, blue) = utils::checksum_to_color(checksum, is_same);
            let checksum_hex = hex::encode(checksum);
            let colored_checksum = checksum_hex.color(colored::Color::TrueColor {
                r: red,
                g: green,
                b: blue,
            });

            let (file_head, file_tail) = utils::split_at_last_segments(file, n);
            let file_display = if options.zero {
                file_head.unwrap_or_default() + &file_tail
            } else {
                escape::escape(&file_head.unwrap_or_default())
                    .dimmed()
                    .to_string()
                    + &file_tail
            };

            let line = if options.tag {
                format!(
                    "{} ({}) = {}",
                    options.algorithm, file_display, colored_checksum
                )
            } else {
                format!("{}  {}", colored_checksum, file_display)
            };
            if options.zero {
                print!("{}\0", line);
            } else {
                println!("{}", line);
            }
        }

        if checksums.iter().any(Option::is_none) {
            anything_failed = true;
        }
        if checksums.len() > 1 {
            if is_same {
                anything_succeeded = true;
            } else {
                anything_group_failed = true;
            }
        }
    }

    if anything_failed {
        std::process::exit(1);
    }
    if options.group_with_check.is_some() {
        if anything_group_failed {
            std::process::exit(1);
        }
        if !anything_succeeded {
            eprintln!(
                "{}",
                "no checksums validated, all groups have only one file".yellow()
            );
        }
    }

    Ok(())
}

fn do_check(options: &Options) -> anyhow::Result<()> {
    let mut anything_succeeded = false;
    let mut anything_failed = false;
    for filepath in &options.files {
        if filepath == "-" {
            for line in std::io::stdin().lines() {
                let line = line?;
                match do_line(options, filepath, &line) {
                    Some(true) => {
                        anything_succeeded = true;
                    }
                    Some(false) => {
                        anything_failed = true;
                    }
                    None => {}
                }
            }
        } else {
            let file = fs_err::File::open(filepath)?;
            for line in std::io::BufReader::new(file).lines() {
                let line = line?;
                match do_line(options, filepath, &line) {
                    Some(true) => {
                        anything_succeeded = true;
                    }
                    Some(false) => {
                        anything_failed = true;
                    }
                    None => {}
                }
            }
        }
    }

    if anything_failed {
        std::process::exit(1);
    } else if !anything_succeeded {
        eprintln!("{}: no checksums validated", "error".red());
        std::process::exit(1);
    }

    Ok(())
}

static BSD_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(&format!(
        r#"^(?P<algorithm>{}) \((?P<filename>.+)\) = (?P<hash>[0-9a-fA-F]{{2}}+)$"#,
        Algorithm::iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join("|")
    ))
    .expect("invalid regex")
});

fn do_line(options: &Options, file: &str, line: &str) -> Option<bool> {
    let mut ret = None;
    let (algorithm, filename, hash) = match parse_line(line) {
        Ok((algorithm, filename, hash)) => (algorithm, filename, hash),
        Err(CheckError::InvalidLine(_)) if options.strict => {
            ret = Some(false);
            return ret;
        }
        Err(CheckError::InvalidLine(_)) if options.warn => {
            eprintln!("{}: {}", file, "invalid line".yellow());
            return ret;
        }
        Err(CheckError::InvalidLine(_)) => {
            return ret;
        }
        Err(_) => unreachable!(),
    };

    let result = process_line(algorithm, options.buffer_size, &filename, &hash);
    match &result {
        Ok(()) => {
            ret = Some(true);
        }
        Err(CheckError::InvalidLine(_)) if options.strict => {
            ret = Some(false);
        }
        Err(CheckError::InvalidLine(_)) if options.warn => {
            eprintln!("{}: {}", &filename, "invalid line".yellow());
        }
        Err(CheckError::ReadFailed(_)) if options.ignore_missing => {}
        Err(_) => {
            ret = Some(false);
        }
    }

    if !options.quiet {
        match result {
            Ok(()) => {
                println!("{}: {}", &filename, "OK".green());
            }
            Err(CheckError::ReadFailed(e)) if options.ignore_missing => {
                eprintln!("{}: {}", &filename, e.to_string().yellow());
            }
            Err(e) => {
                eprintln!("{}: {}", &filename, e.to_string().red());
            }
        }
    }

    ret
}

fn parse_line(line: &str) -> Result<(Algorithm, String, String), CheckError> {
    if let Some((_, hash, filename)) =
        lazy_regex::regex_captures!("^([0-9a-fA-F]{2}+)  (.+)$", line)
    {
        let hash = hash.to_string();
        let filename = filename.to_string();
        let algorithm = Algorithm::SHA256;

        Ok((algorithm, escape::unescape(&filename)?, hash))
    } else if let Some(captures) = BSD_REGEX.captures(line) {
        let algorithm = captures
            .name("algorithm")
            .unwrap()
            .as_str()
            .parse()
            .map_err(|_| {
                CheckError::InvalidLine(format!(
                    "invalid algorithm: {}",
                    captures.name("algorithm").unwrap().as_str()
                ))
            })?;
        let filename = captures.name("filename").unwrap().as_str();
        let hash = captures.name("hash").unwrap().as_str();

        Ok((algorithm, filename.to_string(), hash.to_string()))
    } else {
        Err(CheckError::InvalidLine("pattern not matched".to_string()))
    }
}

fn process_line(
    algorithm: Algorithm,
    buffer_size: usize,

    filename: &str,
    hash: &str,
) -> Result<(), CheckError> {
    let actual = checksum_file(filename, algorithm, buffer_size).map_err(CheckError::ReadFailed)?;
    let expected = hex::decode(hash).expect("unreachable: already validated");
    if actual == expected {
        Ok(())
    } else {
        Err(CheckError::ChecksumMismatch {
            expected: hex::encode(expected),
            actual: hex::encode(actual),
        })
    }
}

#[derive(Debug, thiserror::Error)]
enum CheckError {
    #[error("failed to read file: {0}")]
    ReadFailed(anyhow::Error),

    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("invalid line: {0}")]
    InvalidLine(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
