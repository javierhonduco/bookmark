use std::fs::File;

use std::io::{self, BufRead};

use std::collections::HashMap;

use bitflags::bitflags;

use nix::sys::uio::pread;
use std::convert::TryInto;
use std::os::unix::io::AsRawFd;
use structopt::StructOpt;

const PAGE_SIZE: u64 = 0x1000;

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

bitflags! {
    struct PageMap: u64 {
        const PFN = (1 << 55) - 1;
        const SWAPPED = 1 << 62;
        const PRESENT = 1 << 63;
    }
}

impl PageMap {
    fn is_swapped(self) -> bool {
        self.contains(PageMap::SWAPPED)
    }

    fn is_present(self) -> bool {
        self.contains(PageMap::PRESENT)
    }

    fn pfn(self) -> u64 {
        (self & PageMap::PFN).bits
    }
}

// From https://www.kernel.org/doc/html/latest/admin-guide/mm/pagemap.html?highlight=pagemap
//
// > * Bits 0-54  page frame number (PFN) if present
// > * Bits 0-4   swap type if swapped
// > * Bits 5-54  swap offset if swapped
// > * Bit  55    pte is soft-dirty (see Documentation/vm/soft-dirty.txt)
// > * Bit  56    page exclusively mapped (since 4.2)
// > * Bits 57-60 zero
// > * Bit  61    page is file-page or shared-anon (since 3.5)
// > * Bit  62    page swapped
// > * Bit  63    page present

#[derive(Debug)]
struct MemoryMap {
    low_addr: u64,
    high_addr: u64,
    path: Option<String>,
}

impl MemoryMap {
    fn is_anon(&self) -> bool {
        self.path.is_none()
    }
}

fn memory_maps(pid: u32) -> Vec<MemoryMap> {
    let mut all_maps = Vec::new();
    let maps_file = format!("/proc/{}/maps", pid);
    let maps = File::open(maps_file).unwrap();
    for line in io::BufReader::new(maps).lines() {
        if let Ok(map) = line {
            let splitted_line = map.split_whitespace().collect::<Vec<&str>>();
            let (addr_range, path) = (splitted_line[0], splitted_line.get(5));
            let mut split_addr = addr_range.split("-");
            let (low_addr, high_addr) = (split_addr.next().unwrap(), split_addr.next().unwrap());
            all_maps.push(MemoryMap {
                low_addr: u64::from_str_radix(low_addr, 16).unwrap(),
                high_addr: u64::from_str_radix(high_addr, 16).unwrap(),
                path: path.map(|s| s.to_string()),
            })
        }
    }

    all_maps
}

fn fetch_pagemaps(map: &MemoryMap, pagemaps_file: &File) -> Vec<(u64, PageMap)> {
    let mut result = Vec::new();

    let mut current_addr = map.low_addr;
    let high_addr = map.high_addr;

    while current_addr < high_addr {
        let mut buffer = [0; 8];
        pread(
            pagemaps_file.as_raw_fd(),
            &mut buffer[..],
            (current_addr / PAGE_SIZE * 8).try_into().unwrap(),
        )
        .unwrap();

        result.push((
            current_addr,
            PageMap::from_bits_truncate(u64::from_ne_bytes(buffer)),
        ));

        current_addr += PAGE_SIZE;
    }

    result
}

fn main() {
    let opt = Opt::from_args();

    let pid = opt.pid;
    let mut memory_maps = memory_maps(pid);

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

                let physical_pages = fetch_pagemaps(map, &pagemaps_file);

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
            let pagemaps_path = format!("/proc/{}/pagemap", pid);
            let pagemaps_file = File::open(pagemaps_path).unwrap();
            let mut stats = HashMap::new();

            let anonymous = "anon".to_string();

            #[derive(Debug)]
            struct PageStats {
                swapped: u32,
                present: u32,
                unmapped: u32,
                total: u32,
            }

            impl Default for PageStats {
                fn default() -> Self {
                    Self {
                        swapped: 0,
                        present: 0,
                        unmapped: 0,
                        total: 0,
                    }
                }
            }

            for map in memory_maps.iter_mut() {
                let physical_pages = fetch_pagemaps(map, &pagemaps_file);

                for physical_page in &physical_pages {
                    let path = map.path.as_ref().unwrap_or(&anonymous);
                    let entry = stats.entry(path).or_insert(PageStats::default());
                    if physical_page.1.is_swapped() {
                        entry.swapped += 1;
                    }
                    if physical_page.1.is_present() {
                        entry.present += 1;
                    }
                    if physical_page.1.pfn() == 0 {
                        entry.unmapped += 1;
                    }
                    entry.total += 1;
                }
            }

            let mut sorted_start: Vec<_> = stats.into_iter().collect();
            sorted_start.sort_by(|a, b| a.1.swapped.cmp(&b.1.swapped));

            for (path, count) in sorted_start {
                println!("{} {:?}", path, count);
            }
        }
    }
}
