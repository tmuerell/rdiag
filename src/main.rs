use std::{net::IpAddr, time::Duration};

use clap::{Parser, Subcommand};
use color_eyre::Result;
use colored::Colorize;
use ipnet::Ipv4Net;

#[derive(Debug, Clone, Parser)]
#[command(name = "rdiag", author, about, long_about = None)]
pub struct Opts {
    /// Specify the type of diagnostics you want to run
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Ping {
        addresses: Vec<IpAddr>,
        #[arg(long, default_value = "1000")]
        limit: u64,
    },
    Arp {
        net: Ipv4Net,
        #[arg(default_value = "2")]
        iterations: u32,
    },
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
        Command::Arp { net, iterations } => {
            arp::run(&net, iterations);
        }
    }

    Ok(())
}

mod arp {
    use std::{collections::HashMap, net::Ipv4Addr, time::Duration};

    use ipnet::Ipv4Net;
    use libarp::{client::ArpClient, interfaces::MacAddr};

    pub fn run(net: &Ipv4Net, iterations: u32) {
        eprintln!("Scanning {} for {} passes", net, iterations);
        let mut arp_client = ArpClient::new().expect("Could not get ArpClient");
        let mut l = Vec::new();

        for i in 1..=iterations {
            eprintln!("Pass {i}...");
            for addr in net.hosts() {
                let res = query(&mut arp_client, addr, Some(Duration::from_millis(100)));
                if let Ok((_, mac_addr)) = res {
                    let ip = format!("{addr}");
                    let mac = format!("{mac_addr}");
                    l.push((ip, mac));
                }
            }
            eprintln!("Pass {i} done");
        }

        let mut m: HashMap<String, HashMap<String, u32>> = HashMap::new();
        for (ip, mac) in l {
            m.entry(ip)
                .and_modify(|n| {
                    n.entry(mac.clone()).and_modify(|x| *x += 1).or_insert(1);
                })
                .or_insert({
                    let mut x = HashMap::new();
                    x.insert(mac, 1);
                    x
                });
        }

        for (ip, x) in m {
            if x.len() > 1 {
                println!("{ip}: {x:?}");
            }
        }
    }

    fn query(
        arp_client: &mut ArpClient,
        addr: Ipv4Addr,
        timeout: Option<Duration>,
    ) -> Result<(Ipv4Addr, MacAddr), std::io::Error> {
        let mac = arp_client.ip_to_mac(addr, timeout)?;
        Ok((addr, mac))
    }
}
