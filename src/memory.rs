use bitflags::bitflags;
use nix::sys::uio::pread;
use std::convert::TryInto;
use std::fs::File;
use std::io::{self, BufRead};
use std::os::unix::io::AsRawFd;

const PAGE_SIZE: u64 = 0x1000;

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
pub struct MemoryMap {
    pub low_addr: u64,
    pub high_addr: u64,
    pub path: Option<String>,
}

impl MemoryMap {
    pub fn is_anon(&self) -> bool {
        self.path.is_none()
    }
}

pub fn memory_maps(pid: u32) -> Vec<MemoryMap> {
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

bitflags! {
    pub struct PageMap: u64 {
        const PFN = (1 << 55) - 1;
        const SWAPPED = 1 << 62;
        const PRESENT = 1 << 63;
    }
}

impl PageMap {
    pub fn is_swapped(self) -> bool {
        self.contains(PageMap::SWAPPED)
    }

    pub fn is_present(self) -> bool {
        self.contains(PageMap::PRESENT)
    }

    pub fn pfn(self) -> u64 {
        (self & PageMap::PFN).bits
    }
}

pub fn fetch_pagemaps(map: &MemoryMap, pagemaps_file: &File) -> Vec<(u64, PageMap)> {
    let mut result = Vec::new();

    let low_addr = map.low_addr;
    let high_addr = map.high_addr;

    (low_addr..high_addr)
        .step_by(PAGE_SIZE as usize)
        .for_each(|current_addr| {
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
        });

    result
}
