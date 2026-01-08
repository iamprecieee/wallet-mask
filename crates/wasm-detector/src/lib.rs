use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub struct Match {
    pub value: String,
    pub index: usize,
    pub type_: String,
}

// --- Static Regex Definitions ---

static FULL_ADDRESS_RE: OnceLock<Regex> = OnceLock::new();
static TRUNCATED_RE: OnceLock<Regex> = OnceLock::new();
static ENS_RE: OnceLock<Regex> = OnceLock::new();
static SOL_RE: OnceLock<Regex> = OnceLock::new();
static SOL_TRUNCATED_RE: OnceLock<Regex> = OnceLock::new();

// Transaction hash regex patterns
static ETH_TX_HASH_RE: OnceLock<Regex> = OnceLock::new();
static ETH_TX_TRUNCATED_RE: OnceLock<Regex> = OnceLock::new();
static SOL_TX_SIG_RE: OnceLock<Regex> = OnceLock::new();
static SOL_TX_TRUNCATED_RE: OnceLock<Regex> = OnceLock::new();

fn get_full_address_re() -> &'static Regex {
    FULL_ADDRESS_RE.get_or_init(|| Regex::new(r"\b0x[a-fA-F0-9]{40}\b").unwrap())
}

fn get_truncated_re() -> &'static Regex {
    TRUNCATED_RE
        .get_or_init(|| Regex::new(r"\b0x[a-fA-F0-9]{4,12}(?:\.{3}|…)[a-fA-F0-9]{4,12}\b").unwrap())
}

fn get_ens_re() -> &'static Regex {
    ENS_RE.get_or_init(|| Regex::new(r"(?i)\b[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.eth\b").unwrap())
}

fn get_sol_re() -> &'static Regex {
    SOL_RE.get_or_init(|| Regex::new(r"\b[1-9A-HJ-NP-Za-km-z]{32,44}\b").unwrap())
}

fn get_sol_truncated_re() -> &'static Regex {
    SOL_TRUNCATED_RE.get_or_init(|| {
        Regex::new(r"\b[1-9A-HJ-NP-Za-km-z]{3,10}(?:\.{3}|…)[1-9A-HJ-NP-Za-km-z]{3,10}\b").unwrap()
    })
}

// --- Transaction Hash Regex Getters ---

fn get_eth_tx_hash_re() -> &'static Regex {
    ETH_TX_HASH_RE.get_or_init(|| Regex::new(r"\b0x[a-fA-F0-9]{64}\b").unwrap())
}

fn get_eth_tx_truncated_re() -> &'static Regex {
    ETH_TX_TRUNCATED_RE
        .get_or_init(|| Regex::new(r"\b0x[a-fA-F0-9]{4,12}(?:\.{3}|…)[a-fA-F0-9]{4,12}\b").unwrap())
}

fn get_sol_tx_sig_re() -> &'static Regex {
    SOL_TX_SIG_RE.get_or_init(|| Regex::new(r"\b[1-9A-HJ-NP-Za-km-z]{86,88}\b").unwrap())
}

fn get_sol_tx_truncated_re() -> &'static Regex {
    SOL_TX_TRUNCATED_RE.get_or_init(|| {
        Regex::new(r"\b[1-9A-HJ-NP-Za-km-z]{4,12}(?:\.{3}|…)[1-9A-HJ-NP-Za-km-z]{4,12}\b").unwrap()
    })
}

// --- Helper Functions ---

/// Checks if a given range [start, end) overlaps with any match in the provided list.
fn has_overlap(start: usize, end: usize, matches: &[Match]) -> bool {
    matches.iter().any(|m| {
        let m_end = m.index + m.value.len();
        (start >= m.index && start < m_end) || (m.index >= start && m.index < end)
    })
}

/// Scans text with a regex and collects non-overlapping matches.
fn scan_regex(text: &str, re: &Regex, type_: &str, checks: &[&[Match]]) -> Vec<Match> {
    let mut results = Vec::new();
    for cap in re.find_iter(text) {
        let start = cap.start();
        let end = cap.end();

        let overlaps = checks.iter().any(|list| has_overlap(start, end, list));

        if !overlaps {
            results.push(Match {
                value: cap.as_str().to_string(),
                index: start,
                type_: type_.to_string(),
            });
        }
    }
    results
}

fn is_valid_ens(text: &str) -> bool {
    if !text.ends_with(".eth") {
        return false;
    }
    if text.len() <= 7 {
        return text.len() > 4;
    }
    true
}

// --- Detection Logic ---

// Transaction hash detection (must run before address detection for proper overlap handling)

fn find_eth_tx_hashes(text: &str) -> Vec<Match> {
    scan_regex(text, get_eth_tx_hash_re(), "eth_tx_hash", &[])
}

fn find_eth_tx_truncated(text: &str, existing_matches: &[Match]) -> Vec<Match> {
    scan_regex(
        text,
        get_eth_tx_truncated_re(),
        "eth_tx_truncated",
        &[existing_matches],
    )
}

fn find_sol_tx_signatures(text: &str, existing_matches: &[Match]) -> Vec<Match> {
    let mut matches = Vec::new();

    let full = scan_regex(text, get_sol_tx_sig_re(), "sol_tx_sig", &[existing_matches]);
    matches.extend(full);

    let truncated = scan_regex(
        text,
        get_sol_tx_truncated_re(),
        "sol_tx_truncated",
        &[existing_matches, &matches],
    );
    matches.extend(truncated);

    matches
}

// Address detection

fn find_full_addresses(text: &str, existing_matches: &[Match]) -> Vec<Match> {
    scan_regex(
        text,
        get_full_address_re(),
        "fullAddress",
        &[existing_matches],
    )
}

fn find_truncated_addresses(text: &str, existing_matches: &[Match]) -> Vec<Match> {
    scan_regex(text, get_truncated_re(), "truncated", &[existing_matches])
}

fn find_sol_addresses(text: &str, existing_matches: &[Match]) -> Vec<Match> {
    let mut matches = Vec::new();

    let full = scan_regex(text, get_sol_re(), "sol", &[existing_matches]);
    matches.extend(full);

    let truncated = scan_regex(
        text,
        get_sol_truncated_re(),
        "sol_truncated",
        &[existing_matches, &matches],
    );
    matches.extend(truncated);

    matches
}

fn find_ens_names(text: &str, existing_matches: &[Match]) -> Vec<Match> {
    let mut matches = Vec::new();
    for cap in get_ens_re().find_iter(text) {
        let val = cap.as_str();

        if !is_valid_ens(val) {
            continue;
        }

        let start = cap.start();
        let end = cap.end();

        if !has_overlap(start, end, existing_matches) {
            matches.push(Match {
                value: val.to_string(),
                index: start,
                type_: "ens".to_string(),
            });
        }
    }
    matches
}

// --- Exported API ---

#[wasm_bindgen]
pub fn find_matches(text: &str) -> JsValue {
    // Detect transaction hashes first (longer patterns before shorter ones)
    let mut matches = find_eth_tx_hashes(text);

    let eth_tx_truncated = find_eth_tx_truncated(text, &matches);
    matches.extend(eth_tx_truncated);

    let sol_tx = find_sol_tx_signatures(text, &matches);
    matches.extend(sol_tx);

    // Detect addresses (after tx hashes to avoid partial matches)
    let full_addresses = find_full_addresses(text, &matches);
    matches.extend(full_addresses);

    let truncated = find_truncated_addresses(text, &matches);
    matches.extend(truncated);

    let sol = find_sol_addresses(text, &matches);
    matches.extend(sol);

    let ens = find_ens_names(text, &matches);
    matches.extend(ens);

    matches.sort_by_key(|m| m.index);

    serde_wasm_bindgen::to_value(&matches).unwrap()
}
