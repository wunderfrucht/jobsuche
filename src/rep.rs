//! Response types for the Jobsuche API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Job search response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSearchResponse {
    pub stellenangebote: Vec<JobListing>,
    #[serde(default)]
    pub max_ergebnisse: Option<u64>,
    #[serde(default)]
    pub page: Option<u64>,
    #[serde(default)]
    pub size: Option<u64>,
    /// Facets for filtering (raw HashMap - structure varies)
    #[serde(default)]
    pub facetten: Option<serde_json::Value>,
}

/// Individual job listing in search results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobListing {
    /// Hash ID for the job (may be missing, use refnr instead)
    #[serde(default)]
    pub hash_id: Option<String>,
    /// Reference number (use this for job details)
    pub refnr: String,
    /// Job title/profession
    pub beruf: String,
    /// Job listing title
    #[serde(default)]
    pub titel: Option<String>,
    /// Employer name
    pub arbeitgeber: String,
    /// Publication date (ISO 8601 format: YYYY-MM-DD)
    #[serde(default)]
    pub aktuelle_veroeffentlichungsdatum: Option<String>,
    /// Start date (ISO 8601 format: YYYY-MM-DD)
    #[serde(default)]
    pub eintrittsdatum: Option<String>,
    /// Work location
    pub arbeitsort: WorkLocation,
    /// Modification timestamp
    #[serde(default)]
    pub modifikations_timestamp: Option<String>,
    /// External URL (for external job postings)
    #[serde(default)]
    pub externe_url: Option<String>,
    /// Employer customer number hash (for logos)
    #[serde(default)]
    pub kundennummer_hash: Option<String>,
}

/// Work location information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLocation {
    #[serde(default)]
    pub plz: Option<String>,
    #[serde(default)]
    pub ort: Option<String>,
    #[serde(default)]
    pub strasse: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub land: Option<String>,
    #[serde(default)]
    pub koordinaten: Option<Coordinates>,
    /// Distance from search location in km
    #[serde(default)]
    pub entfernung: Option<String>,
}

/// Geographic coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64,
}

/// Search facets for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Facet {
    #[serde(flatten)]
    pub data: HashMap<String, FacetData>,
}

/// Facet data with counts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FacetData {
    pub counts: HashMap<String, u64>,
    pub max_count: u64,
}

/// Detailed job information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobDetails {
    #[serde(default)]
    pub hash_id: Option<String>,
    #[serde(default)]
    pub refnr: Option<String>,
    #[serde(default)]
    pub titel: Option<String>,
    #[serde(default)]
    pub stellenangebots_art: Option<String>,
    #[serde(default)]
    pub arbeitgeber: Option<String>,
    #[serde(default)]
    pub arbeitgeber_hash_id: Option<String>,
    pub hauptberuf: Option<String>,
    pub beruf: Option<String>,
    #[serde(default)]
    pub branchengruppe: Option<String>,
    #[serde(default)]
    pub branche: Option<String>,
    #[serde(default)]
    pub aktuelle_veroeffentlichungsdatum: Option<String>,
    #[serde(default)]
    pub eintrittsdatum: Option<String>,
    #[serde(default)]
    pub erste_veroeffentlichungsdatum: Option<String>,
    #[serde(default)]
    pub modifikations_timestamp: Option<String>,
    #[serde(default)]
    pub stellenbeschreibung: Option<String>,
    #[serde(default)]
    pub arbeitsorte: Vec<WorkLocation>,
    #[serde(default)]
    pub arbeitgeber_adresse: Option<Address>,
    #[serde(default)]
    pub arbeitszeitmodelle: Vec<String>,
    #[serde(default)]
    pub befristung: Option<String>,
    #[serde(default)]
    pub vertragsdauer: Option<String>,
    #[serde(default)]
    pub uebernahme: Option<bool>,
    #[serde(default)]
    pub betriebsgroesse: Option<String>,
    #[serde(default)]
    pub anzahl_offene_stellen: Option<u32>,
    #[serde(default)]
    pub nur_fuer_schwerbehinderte: Option<bool>,
    #[serde(default)]
    pub fuer_fluechtlinge_geeignet: Option<bool>,
    #[serde(default)]
    pub arbeitgeberdarstellung: Option<String>,
    #[serde(default)]
    pub arbeitgeberdarstellung_url: Option<String>,
    #[serde(default)]
    pub allianzpartner: Option<String>,
    #[serde(default)]
    pub allianzpartner_url: Option<String>,
    #[serde(default)]
    pub verguetung: Option<String>,
    #[serde(default)]
    pub fertigkeiten: Vec<Skill>,
    #[serde(default)]
    pub mobilitaet: Option<Mobility>,
    #[serde(default)]
    pub fuehrungskompetenzen: Option<LeadershipSkills>,
    #[serde(default)]
    pub ist_betreut: Option<bool>,
    #[serde(default)]
    pub ist_google_jobs_relevant: Option<bool>,
    #[serde(default)]
    pub anzeige_anonym: Option<bool>,
}

/// Address information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub land: String,
    pub region: String,
    pub plz: Option<String>,
    pub ort: String,
    #[serde(default)]
    pub strasse: Option<String>,
    #[serde(default)]
    pub strasse_hausnummer: Option<String>,
}

/// Skill/competency requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub hierarchie_name: String,
    #[serde(default)]
    pub auspraegungen: Option<HashMap<String, Vec<String>>>,
}

/// Mobility requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mobility {
    #[serde(default)]
    pub reisebereitschaft: Option<String>,
}

/// Leadership competencies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadershipSkills {
    #[serde(default)]
    pub hat_vollmacht: Option<bool>,
    #[serde(default)]
    pub hat_budgetverantwortung: Option<bool>,
}

// Enums for type-safe parameters

/// Employment type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum Angebotsart {
    /// Regular employment (ARBEIT)
    Arbeit = 1,
    /// Self-employment (SELBSTAENDIGKEIT)
    Selbstaendigkeit = 2,
    /// Apprenticeship/Dual study (AUSBILDUNG/Duales Studium)
    Ausbildung = 4,
    /// Internship/Trainee (Praktikum/Trainee)
    PraktikumTrainee = 34,
}

impl Angebotsart {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Arbeit => "1",
            Self::Selbstaendigkeit => "2",
            Self::Ausbildung => "4",
            Self::PraktikumTrainee => "34",
        }
    }
}

/// Contract type (befristung)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum Befristung {
    /// Fixed-term contract (befristet)
    Befristet = 1,
    /// Permanent contract (unbefristet)
    Unbefristet = 2,
}

impl Befristung {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Befristet => "1",
            Self::Unbefristet => "2",
        }
    }
}

/// Working time models
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Arbeitszeit {
    /// Full-time (VOLLZEIT)
    Vollzeit,
    /// Part-time (TEILZEIT)
    Teilzeit,
    /// Shift/Night/Weekend work (SCHICHT_NACHTARBEIT_WOCHENENDE)
    SchichtNachtarbeitWochenende,
    /// Home office/Remote (HEIM_TELEARBEIT)
    HeimTelearbeit,
    /// Mini job (MINIJOB)
    Minijob,
}

impl Arbeitszeit {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Vollzeit => "vz",
            Self::Teilzeit => "tz",
            Self::SchichtNachtarbeitWochenende => "snw",
            Self::HeimTelearbeit => "ho",
            Self::Minijob => "mj",
        }
    }
}
