use std::fs::File;

use nix::unistd::geteuid;
use structopt::StructOpt;

mod memory;
mod stats;

#[macro_use]
extern crate more_asserts;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    pid: u32,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    List {
        #[structopt(short, long)]
        swapped: bool,
        #[structopt(short, long)]
        present: bool,
        #[structopt(short, long)]
        anon: bool,
    },
    Stats {},
}

fn main() {
    if !geteuid().is_root() {
        eprintln!("root is required");
        return;
    }

    let opt = Opt::from_args();

    let pid = opt.pid;
    let mut memory_maps = memory::memory_maps(pid);

    match opt.cmd {
        Command::List {
            swapped,
            present,
            anon,
        } => {
            let pagemaps_path = format!("/proc/{}/pagemap", pid);
            let pagemaps_file = File::open(pagemaps_path).unwrap();

            for map in memory_maps.iter_mut() {
                if anon && !map.is_anon() {
                    continue;
                }

                let physical_pages = memory::fetch_pagemaps(map, &pagemaps_file);

                for physical_page in &physical_pages {
                    if swapped && !physical_page.1.is_swapped() {
                        continue;
                    }
                    if present && !physical_page.1.is_present() {
                        continue;
                    }

                    println!(
                        "{:x} {:x} {} {:?}",
                        physical_page.0,
                        physical_page.1.pfn(),
                        physical_page.1.is_swapped(),
                        map.path
                    );
                }
            }
        }
        Command::Stats {} => {
            let mut sorted_start: Vec<_> = stats::page_stats(pid).into_iter().collect();
            sorted_start.sort_by(|a, b| a.1.swapped.cmp(&b.1.swapped));

            for (path, count) in sorted_start {
                println!("{} {:?}", path, count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use nix::unistd::getpid;

    use super::*;
    use crate::memory::*;
    use crate::stats::*;
    use std::mem;

    #[test]
    fn test_pagemap_size() {
        assert_eq!(mem::size_of::<PageMap>(), mem::size_of::<u64>());
    }

    #[test]
    fn test_pagemap_members() {
        assert_eq!(PageMap::SWAPPED.bits(), 0x4000000000000000);
        assert_eq!(PageMap::PRESENT.bits(), 0x8000000000000000);
        assert_eq!(PageMap::PFN.bits(), 0x7fffffffffffff);
    }

    #[test]
    fn test_stats() {
        fn self_stats() -> PageStats {
            page_stats(getpid().as_raw() as u32)
                .get("anon")
                .unwrap()
                .clone()
        }

        let before = self_stats();
        let mut vec: Vec<u64> = Vec::with_capacity(100_000);
        let after_alloc = self_stats();
        vec.fill(1);
        let after_touch = self_stats();

        // As the pages haven't been touched yet, they are not mapped
        // at this point
        assert_eq!(after_alloc.unmapped, after_alloc.total);
        assert_gt!(after_alloc.total, before.total);
        // After touching them, they get mapped or present
        assert_gt!(after_touch.present, before.present);
    }
}
