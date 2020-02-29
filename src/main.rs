use crate::envfile::EnvRow;
use crate::manifests::{config_map, Metadata};
use crate::SubCommand::ConfigMap;
use clap::Clap;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Error, Write};

mod envfile;
mod manifests;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "0.1", author = "William Rudenmalm <me@whn.se>")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// A help message for the Test subcommand
    #[clap(name = "configmap")]
    ConfigMap(ConfigMapCommand),
}

/// A subcommand for controlling testing
#[derive(Clap)]
struct ConfigMapCommand {
    input: String,
    #[clap(short, long)]
    output: Option<String>,
}

fn read_env_from_bufread(mut br: Box<dyn BufRead>) -> Result<Vec<envfile::EnvRow>, Error> {
    let mut buf = String::new();
    let mut res = Vec::new();
    loop {
        buf.clear();
        match br.read_line(&mut buf) {
            Ok(n) if n > 0 => {
                if let Some(row) = envfile::parse_line(buf.as_bytes()) {
                    res.push(row)
                }
            }
            Ok(0) => return Ok(res),
            Err(e) => return Err(e),
            _ => panic!("Impossible"),
        }
    }
}

fn bufread_from_path(path: String) -> Result<Box<dyn BufRead>, io::Error> {
    let b: Box<dyn BufRead> = match path.as_str() {
        "-" => Box::new(BufReader::new(io::stdin())),
        x => Box::new(BufReader::new(File::open(x)?)),
    };
    Ok(b)
}

fn output_for(path: Option<String>) -> Result<Box<dyn Write>, io::Error> {
    Ok(match path.filter(|x| x == "-") {
        None => Box::new(io::stdout()) as Box<dyn Write>,
        Some(x) => Box::new(BufWriter::new(File::create(x.clone())?)) as Box<dyn Write>,
    })
}

fn read_env(handle: Box<dyn BufRead>) -> Result<Vec<(String, String)>, io::Error> {
    let lines = read_env_from_bufread(handle)?;
    let v: Vec<(String, String)> = lines
        .into_iter()
        .filter_map(|x| match x {
            EnvRow::Env(k, v) => Some((k, v)),
            _ => None,
        })
        .collect();
    Ok(v)
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        ConfigMap(cmd) => {
            let input = bufread_from_path(cmd.input.clone()).unwrap();
            let mut output = output_for(cmd.output.clone()).unwrap();
            let input_path = cmd.input.as_str().clone();

            let start = input_path.rfind('/').unwrap_or(0);
            let end = input_path.rfind('.').unwrap_or(input_path.len());
            let name = &input_path[start..end];
            let envs = read_env(input).unwrap();
            let v = config_map(
                Metadata::name_only(if name == "-" { "unnamed" } else { name }),
                envs.iter().map(|x| x.to_owned()),
            );
            let json = serde_json::to_string_pretty(&v).unwrap();
            let n = output.write(json.as_bytes()).unwrap();
            if n == 0 {
                eprintln!("Unable to write result")
            }
        }
    }
}
