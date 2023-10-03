use std::{net::IpAddr, time::Duration};

use clap::{Parser, Subcommand};
use color_eyre::Result;
use colored::Colorize;

#[derive(Debug, Clone, Parser)]
#[command(name = "rdiag", author, about, long_about = None)]
pub struct Opts {
    /// Specify the type of diagnostics you want to run
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Ping { addresses: Vec<IpAddr>, 
        #[arg(long, default_value = "1000")]
        limit : u64 },
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    match opts.command {
        Command::Ping { addresses, limit } => {
            for i in 0..limit {
                println!("----- {:5} -----", format!("{}", i).magenta());
                for addr in &addresses {
                    let data = [1, 2, 3, 4]; // ping data
                    let timeout = Duration::from_secs(1);
                    let options = ping_rs::PingOptions {
                        ttl: 128,
                        dont_fragment: false,
                    };
                    let result = ping_rs::send_ping(&addr, timeout, &data, Some(&options));
                    match result {
                        Ok(reply) => println!(
                            "  {} {}: bytes={} time={}ms TTL={}",
                            reply.address,
                            "OK".green(),
                            data.len(),
                            reply.rtt,
                            options.ttl
                        ),
                        Err(e) => println!("  {} {}: {:?}", &addr, "NOK".red(), e),
                    }
                }
            }
        }
    }

    Ok(())
}
