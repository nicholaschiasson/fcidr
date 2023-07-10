use std::{
    error::Error,
    io::{stdin, IsTerminal},
};

use clap::{CommandFactory, Parser, Subcommand};
use fcidr::{Cidr, Fcidr};

#[derive(Debug, Parser)]
#[command(about, author, version, long_about = None)]
struct Cli {
    /// The input CIDR range and first operand to the computation. If omitted,
    /// input is taken from stdin. In this way, multiple computations can be
    /// chained together.
    cidr: Option<Cidr>,
    #[command(subcommand)]
    command: FcidrCommand,
}

#[derive(Debug, Subcommand)]
enum FcidrCommand {
    /// Compute the complement of the input CIDR(s)
    #[command(visible_alias = "!", visible_alias = "not")]
    Complement,
    /// Compute the set difference between the input CIDR(s) and another CIDR
    #[command(
        visible_alias = "-",
        visible_alias = "exclude",
        visible_alias = "minus"
    )]
    Difference {
        /// The second CIDR range operand for the difference function
        cidr: Cidr,
    },
    #[command(visible_alias = "+", visible_alias = "include", visible_alias = "plus")]
    /// Compute the set union of the input CIDR(s) and another CIDR
    Union {
        /// The second CIDR range operand for the union function
        cidr: Cidr,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut fcidr: Fcidr = if let Some(cidr) = cli.cidr {
        Fcidr::new(cidr)
    } else {
        if stdin().is_terminal() {
            Cli::command().print_help().unwrap();
            ::std::process::exit(2);
        }
        stdin().lines().fold(
            Ok(Fcidr::default()),
            |fcidr: Result<Fcidr, Box<dyn Error>>, l| {
                if let Ok(mut fcidr) = fcidr {
                    fcidr.union(l?.parse()?);
                    return Ok(fcidr);
                }
                fcidr
            },
        )?
    };

    match cli.command {
        FcidrCommand::Complement => fcidr.complement(),
        FcidrCommand::Difference { cidr } => fcidr.difference(cidr),
        FcidrCommand::Union { cidr } => fcidr.union(cidr),
    };

    for cidr in fcidr {
        println!("{cidr}");
    }

    Ok(())
}
