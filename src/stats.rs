use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;

use crate::memory;

#[derive(Debug, Clone, Serialize)]
#[derive(Default)]
pub struct PageStats {
    pub swapped: u32,
    pub present: u32,
    pub unmapped: u32,
    pub total: u32,
}



pub fn page_stats(pid: u32) -> HashMap<String, PageStats> {
    let mut memory_maps = memory::memory_maps(pid);

    let pagemaps_path = format!("/proc/{}/pagemap", pid);
    let pagemaps_file = File::open(pagemaps_path).unwrap();
    let mut stats = HashMap::new();

    let anonymous = "anon".to_string();

    for map in memory_maps.iter_mut() {
        let physical_pages = memory::fetch_pagemaps(map, &pagemaps_file);

        for physical_page in &physical_pages {
            let path = map.path.as_ref().unwrap_or(&anonymous);
            let entry = stats
                .entry(path.to_string())
                .or_insert(PageStats::default());
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

    stats
}
