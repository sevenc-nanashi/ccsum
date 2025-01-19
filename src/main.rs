use crate::digest_ext::HashExt;
use clap::{CommandFactory, Parser};
use colored::Colorize;
use palette::IntoColor;
use std::io::{BufRead, Read};
use strum::IntoEnumIterator;
mod digest_ext;
mod escape;

#[derive(
    Debug, Copy, Clone, clap::ValueEnum, strum::Display, strum::EnumString, strum::EnumIter,
)]
enum Algorithm {
    #[clap(name = "md5")]
    MD5,
    #[clap(name = "sha1")]
    SHA1,
    #[clap(name = "sha256")]
    SHA256,
    #[clap(name = "sha512")]
    SHA512,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
enum Group {
    Dir,
    Basename,
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
    #[clap(long, default_value = "false")]
    tag: bool,

    /// end each output line with a NULL character instead of newline, and disable file name
    /// escaping.
    #[clap(short, long, default_value = "false")]
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

    /// group output by specified method.
    #[clap(short, long)]
    group: Option<Group>,

    /// colorize the output, even if stdout is not a tty.
    #[clap(alias = "C", long, default_value = "false")]
    color: bool,

    /// disable colorized output.
    #[clap(long, default_value = "false")]
    no_color: bool,

    /// the files to generate the checksum for.
    files: Vec<String>,
}

fn checksum_bytes(data: &[u8], algorithm: Algorithm) -> Vec<u8> {
    match algorithm {
        Algorithm::MD5 => md5::Md5::hash(data),
        Algorithm::SHA1 => sha1::Sha1::hash(data),
        Algorithm::SHA256 => sha2::Sha256::hash(data),
        Algorithm::SHA512 => sha2::Sha512::hash(data),
    }
}

fn checksum_file(file: &str, algorithm: Algorithm) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut file = fs_err::File::open(file)?;
    file.read_to_end(&mut data)?;
    Ok(checksum_bytes(&data, algorithm))
}

fn checksum_stdin(algorithm: Algorithm) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    std::io::stdin().read_to_end(&mut data)?;
    Ok(checksum_bytes(&data, algorithm))
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

    match options.group {
        Some(Group::Dir) => {
            options.files.sort_by(|a, b| {
                let a = std::path::Path::new(a);
                let b = std::path::Path::new(b);
                a.parent()
                    .unwrap_or_else(|| std::path::Path::new(""))
                    .cmp(b.parent().unwrap_or_else(|| std::path::Path::new("")))
            });
        }
        Some(Group::Basename) => {
            options.files.sort_by(|a, b| {
                let a = std::path::Path::new(a);
                let b = std::path::Path::new(b);
                a.file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new(""))
                    .cmp(b.file_name().unwrap_or_else(|| std::ffi::OsStr::new("")))
            });
        }
        None => {}
    }

    if options.check {
        do_check(&options)?;
    } else {
        do_checksum(&options)?;
    }

    Ok(())
}

fn do_checksum(options: &Options) -> anyhow::Result<()> {
    let mut anything_failed = false;
    for file in &options.files {
        let checksum = if file == "-" {
            checksum_stdin(options.algorithm)
        } else {
            checksum_file(file, options.algorithm)
        };
        let checksum = match checksum {
            Ok(checksum) => checksum,
            Err(e) => {
                eprintln!("{}: {}", file, e.to_string().red());
                anything_failed = true;
                continue;
            }
        };

        let hue = ((checksum[0] as f32) * 256.0 + checksum[1] as f32) / (256.0 * 256.0) * 360.0;
        let color = palette::oklch::Oklch::new(0.7, 0.4, hue);
        let rgb: palette::Srgb<f32> = color.into_color();
        let rgb: palette::Srgb<u8> = rgb.into_format();
        let checksum_hex = hex::encode(checksum);
        let colored_checksum = checksum_hex.color(colored::Color::TrueColor {
            r: rgb.red,
            g: rgb.green,
            b: rgb.blue,
        });

        let line = if options.tag {
            format!(
                "{} ({}) = {}",
                options.algorithm,
                escape::escape(file),
                colored_checksum
            )
        } else {
            format!("{}  {}", colored_checksum, escape::escape(file))
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

    let result = process_line(algorithm, &filename, &hash);
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

fn process_line(algorithm: Algorithm, filename: &str, hash: &str) -> Result<(), CheckError> {
    let actual = checksum_file(filename, algorithm).map_err(CheckError::ReadFailed)?;
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
