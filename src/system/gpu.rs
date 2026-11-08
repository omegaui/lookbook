//! Names the graphics cards. Each card in /sys/class/drm has a PCI ID, which the
//! PCI ID database in the Flatpak runtime turns into a name.

use std::fs;

const DRM_FOLDER: &str = "/sys/class/drm";
const PCI_DATABASE: &str = "/usr/share/hwdata/pci.ids";
/// Vendors the app must recognise even for a card newer than the database,
/// so an NVIDIA card is never mistaken for one that needs no extra driver.
const KNOWN_VENDORS: [(&str, &str); 3] = [("10de", "NVIDIA"), ("1002", "AMD"), ("8086", "Intel")];

/// Returns names like "NVIDIA GeForce RTX 5050 Max-Q / Mobile", one per card.
pub fn detect_gpus() -> Vec<String> {
    let database = fs::read_to_string(PCI_DATABASE).unwrap_or_default();

    card_pci_ids()
        .iter()
        .map(|(vendor, device)| gpu_name(&database, vendor, device))
        .collect()
}

/// The vendor and device IDs of each card, for example ("10de", "2dd8").
fn card_pci_ids() -> Vec<(String, String)> {
    let Ok(entries) = fs::read_dir(DRM_FOLDER) else {
        return Vec::new();
    };

    let mut card_folders: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| is_card_folder(name))
        .collect();
    card_folders.sort();

    card_folders
        .iter()
        .filter_map(|card| fs::read_to_string(format!("{DRM_FOLDER}/{card}/device/uevent")).ok())
        .filter_map(|uevent| parse_pci_id(&uevent))
        .collect()
}

/// "card0" is a card; "card0-HDMI-A-1" is one of its outputs.
fn is_card_folder(name: &str) -> bool {
    name.strip_prefix("card")
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
}

/// Reads "PCI_ID=10DE:2DD8" from a card's uevent file.
fn parse_pci_id(uevent: &str) -> Option<(String, String)> {
    let value = uevent.lines().find_map(|line| line.strip_prefix("PCI_ID="))?;
    let (vendor, device) = value.split_once(':')?;
    Some((vendor.to_lowercase(), device.to_lowercase()))
}

/// Builds a short name from the database's vendor and device lines.
fn gpu_name(database: &str, vendor_id: &str, device_id: &str) -> String {
    let Some((vendor, device)) = lookup(database, vendor_id, device_id) else {
        return unknown_card_name(vendor_id, device_id);
    };

    let vendor = short_name(&vendor);
    let device = short_name(&device);
    if device.starts_with(&vendor) {
        return device;
    }
    format!("{vendor} {device}")
}

/// A card the database does not know yet, for example "NVIDIA card 10de:2f04".
fn unknown_card_name(vendor_id: &str, device_id: &str) -> String {
    let vendor = KNOWN_VENDORS
        .iter()
        .find(|(id, _)| *id == vendor_id)
        .map(|(_, name)| *name);
    match vendor {
        Some(vendor) => format!("{vendor} card {vendor_id}:{device_id}"),
        None => format!("{vendor_id}:{device_id}"),
    }
}

/// Finds "10de  NVIDIA Corporation" and, below it, "\t2dd8  GB207M [GeForce …]".
fn lookup(database: &str, vendor_id: &str, device_id: &str) -> Option<(String, String)> {
    let mut vendor_name: Option<String> = None;

    for line in database.lines() {
        if let Some(rest) = line.strip_prefix(vendor_id) {
            if rest.starts_with("  ") {
                vendor_name = Some(rest.trim().to_owned());
                continue;
            }
        }

        let in_vendor_block = vendor_name.is_some();
        if !in_vendor_block {
            continue;
        }
        if !line.starts_with('\t') && !line.starts_with('#') && !line.is_empty() {
            return None;
        }
        if let Some(rest) = line.strip_prefix('\t').and_then(|line| line.strip_prefix(device_id)) {
            if rest.starts_with("  ") {
                return Some((vendor_name?, rest.trim().to_owned()));
            }
        }
    }

    None
}

/// "GB207M [GeForce RTX 5050]" becomes "GeForce RTX 5050";
/// "NVIDIA Corporation" becomes "NVIDIA"; "Advanced Micro Devices, Inc. [AMD/ATI]" becomes "AMD/ATI".
fn short_name(name: &str) -> String {
    if let Some((_, inside)) = name.split_once('[') {
        return inside.trim_end_matches(']').trim().to_owned();
    }
    name.trim_end_matches(" Corporation").trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATABASE: &str = "\
# comment
10de  NVIDIA Corporation
\t2dd8  GB207M [GeForce RTX 5050 Max-Q / Mobile]
\t2dd9  Other
1002  Advanced Micro Devices, Inc. [AMD/ATI]
\t15bf  Phoenix1
8086  Intel Corporation
\ta788  Raptor Lake-S UHD Graphics
";

    #[test]
    fn names_cards_from_the_database() {
        assert_eq!(gpu_name(DATABASE, "10de", "2dd8"), "NVIDIA GeForce RTX 5050 Max-Q / Mobile");
        assert_eq!(gpu_name(DATABASE, "8086", "a788"), "Intel Raptor Lake-S UHD Graphics");
        assert_eq!(gpu_name(DATABASE, "1002", "15bf"), "AMD/ATI Phoenix1");
    }

    #[test]
    fn falls_back_to_the_ids() {
        assert_eq!(gpu_name(DATABASE, "abcd", "ffff"), "abcd:ffff");
    }

    #[test]
    fn a_card_newer_than_the_database_keeps_its_vendor() {
        assert_eq!(gpu_name(DATABASE, "10de", "ffff"), "NVIDIA card 10de:ffff");
        assert_eq!(gpu_name("", "10de", "2dd8"), "NVIDIA card 10de:2dd8");
    }

    #[test]
    fn reads_pci_ids_and_card_folders() {
        assert_eq!(parse_pci_id("DRIVER=i915\nPCI_ID=8086:A788\n"), Some(("8086".into(), "a788".into())));
        assert!(is_card_folder("card1"));
        assert!(!is_card_folder("card1-HDMI-A-1"));
        assert!(!is_card_folder("renderD128"));
    }
}
