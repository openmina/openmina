use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

use anyhow::{bail, format_err, Result};
use binprot::{BinProtRead, BinProtWrite};
use clap::{Parser, ValueEnum};
use mina_p2p_messages::{
    gossip::GossipNetMessageV2,
    rpc_kernel::{DebuggerMessage, Message},
    utils::{FromBinProtStream, Greedy},
    v2::{
        BlockchainSnarkBlockchainStableV2, MinaBlockBlockStableV2,
        NetworkPoolSnarkPoolDiffVersionedStableV2, NetworkPoolTransactionPoolDiffVersionedStableV2,
    },
};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Parser, Debug)]
struct Cli {
    /// Input Mina type
    #[arg(short, long)]
    _type: String,

    /// Output Mina type
    #[arg(short = 'T', long)]
    output_type: Option<String>,

    /// Input file format
    #[arg(short, long)]
    format: Option<FileFormat>,

    /// Output file format
    #[arg(short = 'F', long)]
    output_format: Option<FileFormat>,

    /// Output file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Pretty-print, if supported
    #[arg(short, long)]
    pretty: bool,

    /// Print list of supported input types
    #[arg(long)]
    print_types: bool,

    /// Print supported type conversions
    #[arg(long)]
    print_conversions: bool,

    /// Input file
    input: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
enum FileFormat {
    BinProt,
    BinProtStream,
    JSON,
}

impl Cli {
    fn process_input<I, O>(&self, input: &Option<PathBuf>) -> Result<()>
    where
        I: BinProtRead + DeserializeOwned,
        O: BinProtWrite + Serialize + TryFrom<I>,
    {
        let format = FileFormat::new(&self.format, input)
            .ok_or_else(|| format_err!("Cannot determine input format"))?;

        let value: I = match input.as_ref() {
            Some(file) => format.read(&mut File::open(file)?)?,
            None => format.read(&mut io::stdin())?,
        };

        let value: O = value
            .try_into()
            .map_err(|_| format_err!("Failed to convert Mina type"))?;

        let format = FileFormat::new(&self.output_format, &self.output)
            .ok_or_else(|| format_err!("Cannot determine output format"))?;

        match self.output.as_ref() {
            Some(file) => format.write(&value, &mut File::create(file)?, false, self.pretty)?,
            None => format.write(&value, &mut io::stdout(), true, self.pretty)?,
        };

        Ok(())
    }

    fn format(&self) -> Result<()> {
        let formatter = formatter(&self._type)
            .ok_or_else(|| format_err!("Cannot find formatter for type `{}`", self._type))?;
        formatter(self, &self.input)?;
        Ok(())
    }

    fn print_formats(&self) -> Result<()> {
        formatters().into_iter().for_each(|f| {
            println!("  {}: {}", f.in_type, f.help);
        });
        Ok(())
    }

    fn print_converters(&self) -> Result<()> {
        converters().into_iter().for_each(|f| {
            println!("  {}", f.in_type);
            f.out_types.into_iter().for_each(|(out_type, help, _)| {
                println!("      {}: {}", out_type, help);
            });
        });
        Ok(())
    }

    fn convert(&self, output_type: &str) -> Result<()> {
        let converter = converter(&self._type, output_type).ok_or_else(|| {
            format_err!(
                "Cannot find converter from `{}` to `{}`",
                self._type,
                output_type
            )
        })?;
        converter(self, &self.input)?;
        Ok(())
    }

    fn main(&self) -> Result<()> {
        if self.print_types {
            self.print_formats()
        } else if self.print_conversions {
            self.print_converters()
        } else if let Some(output_type) = &self.output_type {
            self.convert(output_type)
        } else {
            self.format()
        }
    }
}

impl FileFormat {
    fn new(format: &Option<FileFormat>, input: &Option<PathBuf>) -> Option<FileFormat> {
        format.or_else(|| {
            input.as_ref().and_then(|f| match f.extension() {
                Some(ext) if ext == "bin" || ext == "binprot" => Some(FileFormat::BinProt),
                Some(ext) if ext == "json" => Some(FileFormat::JSON),
                _ => None,
            })
        })
    }

    fn read<T: FromBinProtStream + DeserializeOwned, R: Read>(&self, r: &mut R) -> Result<T> {
        Ok(match self {
            FileFormat::BinProt => T::binprot_read(r)?,
            FileFormat::BinProtStream => T::read_from_stream(r)?,
            FileFormat::JSON => serde_json::from_reader(r)?,
        })
    }

    fn write<T: BinProtWrite + Serialize, W: Write>(
        &self,
        value: &T,
        w: &mut W,
        stdout: bool,
        pretty: bool,
    ) -> Result<()> {
        match (self, stdout, pretty) {
            (FileFormat::BinProt, false, _) => value.binprot_write(w)?,
            (FileFormat::BinProt, true, _) => {
                bail!("stdout is not a best output for binary data...")
            }
            (FileFormat::BinProtStream, _, _) => bail!("unsupported output format"),
            (FileFormat::JSON, _, false) => serde_json::to_writer(w, value)?,
            (FileFormat::JSON, _, true) => serde_json::to_writer_pretty(w, value)?,
        }
        Ok(())
    }
}
type Handler = Box<(dyn Fn(&Cli, &Option<PathBuf>) -> Result<()>)>;

struct Formatter {
    in_type: &'static str,
    help: String,
    handler: Handler,
}

macro_rules! formatter {
    ($type:expr, $ty:ty) => {
        Formatter {
            in_type: $type.into(),
            help: format!("Outputs type `{}` using specific format", stringify!($ty)),
            handler: Box::new(|cli, input| {
                cli.process_input::<$ty, $ty>(input)?;
                Ok(())
            }),
        }
    };
}

fn formatters() -> Vec<Formatter> {
    vec![
        formatter!("gossip", GossipNetMessageV2),
        formatter!("blockchain", BlockchainSnarkBlockchainStableV2),
        formatter!("rpc-generic-debugger", DebuggerMessage<Greedy>),
    ]
}

fn formatter(in_type: &str) -> Option<Handler> {
    formatters()
        .into_iter()
        .find(|h| h.in_type == in_type)
        .map(|h| h.handler)
}

struct Converter {
    in_type: &'static str,
    out_types: Vec<(&'static str, String, Handler)>,
}

macro_rules! converter {
    ($name:expr, $ty:ty => $(($outname:expr, $outty:ty)),* $(,)?) => {
        Converter {
            in_type: $name.into(),
            out_types: vec![$((
                $outname,
                format!(
                    "Converts type `{}` into `{}`",
                    stringify!($ty),
                    stringify!($outty)
                ),
                Box::new(|cli, input| {
                    cli.process_input::<$ty, $outty>(input)?;
                    Ok(())
                }),
            )),*],
        }
    };
}

fn converters() -> Vec<Converter> {
    vec![
        converter!("gossip", GossipNetMessageV2 =>
                   ("new-state", MinaBlockBlockStableV2),
                   ("snark-pool-diff", NetworkPoolSnarkPoolDiffVersionedStableV2),
                   ("tx-pool-diff", NetworkPoolTransactionPoolDiffVersionedStableV2),
        ),
        converter!("rpc-generic-debugger", DebuggerMessage<Greedy> => ("rpc-generic", Message<Greedy>)),
    ]
}

fn converter(in_type: &str, out_type: &str) -> Option<Handler> {
    let converter = converters().into_iter().find(|h| h.in_type == in_type)?;
    converter
        .out_types
        .into_iter()
        .find(|h| h.0 == out_type)
        .map(|h| h.2)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.main()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::FileFormat;

    #[test]
    fn get_format() {
        assert!(FileFormat::new(&None, &None).is_none());
        assert!(FileFormat::new(&None, &Some("qqq".into())).is_none());
        assert!(FileFormat::new(&None, &Some("qqq.ext".into())).is_none());
        assert_eq!(
            FileFormat::new(&None, &Some("qqq.bin".into())),
            Some(FileFormat::BinProt)
        );
        assert_eq!(
            FileFormat::new(&None, &Some("qqq.json".into())),
            Some(FileFormat::JSON)
        );
        assert_eq!(
            FileFormat::new(&Some(FileFormat::JSON), &None),
            Some(FileFormat::JSON)
        );
        assert_eq!(
            FileFormat::new(&Some(FileFormat::BinProt), &Some("qqq.ext".into())),
            Some(FileFormat::BinProt)
        );
    }
}
