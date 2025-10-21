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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angebotsart_as_str() {
        assert_eq!(Angebotsart::Arbeit.as_str(), "1");
        assert_eq!(Angebotsart::Selbstaendigkeit.as_str(), "2");
        assert_eq!(Angebotsart::Ausbildung.as_str(), "4");
        assert_eq!(Angebotsart::PraktikumTrainee.as_str(), "34");
    }

    #[test]
    fn test_befristung_as_str() {
        assert_eq!(Befristung::Befristet.as_str(), "1");
        assert_eq!(Befristung::Unbefristet.as_str(), "2");
    }

    #[test]
    fn test_arbeitszeit_as_str() {
        assert_eq!(Arbeitszeit::Vollzeit.as_str(), "vz");
        assert_eq!(Arbeitszeit::Teilzeit.as_str(), "tz");
        assert_eq!(Arbeitszeit::SchichtNachtarbeitWochenende.as_str(), "snw");
        assert_eq!(Arbeitszeit::HeimTelearbeit.as_str(), "ho");
        assert_eq!(Arbeitszeit::Minijob.as_str(), "mj");
    }

    #[test]
    fn test_job_search_response_deserialization() {
        let json = r#"{
            "stellenangebote": [
                {
                    "refnr": "12345-TEST-S",
                    "beruf": "Software Developer",
                    "arbeitgeber": "Test Corp",
                    "arbeitsort": {
                        "ort": "Berlin",
                        "plz": "10115"
                    }
                }
            ],
            "maxErgebnisse": 1,
            "page": 0,
            "size": 10
        }"#;

        let response: JobSearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.stellenangebote.len(), 1);
        assert_eq!(response.max_ergebnisse, Some(1));
        assert_eq!(response.page, Some(0));
        assert_eq!(response.size, Some(10));
    }

    #[test]
    fn test_job_listing_deserialization() {
        let json = r#"{
            "refnr": "10001-1234567-S",
            "beruf": "Rust Developer",
            "arbeitgeber": "Awesome Company",
            "aktuelleVeroeffentlichungsdatum": "2025-10-21",
            "arbeitsort": {
                "ort": "Munich",
                "plz": "80331",
                "region": "Bayern"
            }
        }"#;

        let listing: JobListing = serde_json::from_str(json).unwrap();
        assert_eq!(listing.refnr, "10001-1234567-S");
        assert_eq!(listing.beruf, "Rust Developer");
        assert_eq!(listing.arbeitgeber, "Awesome Company");
        assert_eq!(
            listing.aktuelle_veroeffentlichungsdatum,
            Some("2025-10-21".to_string())
        );
    }

    #[test]
    fn test_work_location_deserialization() {
        let json = r#"{
            "ort": "Berlin",
            "plz": "10115",
            "region": "Berlin",
            "land": "Deutschland",
            "koordinaten": {
                "lat": 52.52,
                "lon": 13.405
            }
        }"#;

        let location: WorkLocation = serde_json::from_str(json).unwrap();
        assert_eq!(location.ort, Some("Berlin".to_string()));
        assert_eq!(location.plz, Some("10115".to_string()));
        assert!(location.koordinaten.is_some());

        let coords = location.koordinaten.unwrap();
        assert_eq!(coords.lat, 52.52);
        assert_eq!(coords.lon, 13.405);
    }

    #[test]
    fn test_job_details_deserialization() {
        let json = r#"{
            "refnr": "10001-TEST-S",
            "titel": "Senior Rust Engineer",
            "arbeitgeber": "Tech GmbH",
            "stellenbeschreibung": "Great opportunity...",
            "arbeitszeitmodelle": ["VOLLZEIT", "TEILZEIT"],
            "befristung": "unbefristet",
            "arbeitsorte": [
                {
                    "ort": "Hamburg",
                    "plz": "20095"
                }
            ],
            "fertigkeiten": [
                {
                    "hierarchieName": "Programming",
                    "auspraegungen": {
                        "languages": ["Rust", "Go"]
                    }
                }
            ]
        }"#;

        let details: JobDetails = serde_json::from_str(json).unwrap();
        assert_eq!(details.refnr, Some("10001-TEST-S".to_string()));
        assert_eq!(details.titel, Some("Senior Rust Engineer".to_string()));
        assert_eq!(details.arbeitszeitmodelle.len(), 2);
        assert_eq!(details.arbeitsorte.len(), 1);
        assert_eq!(details.fertigkeiten.len(), 1);
    }

    #[test]
    fn test_job_details_optional_fields() {
        let json = r#"{
            "refnr": "10001-MINIMAL-S"
        }"#;

        let details: JobDetails = serde_json::from_str(json).unwrap();
        assert_eq!(details.refnr, Some("10001-MINIMAL-S".to_string()));
        assert_eq!(details.titel, None);
        assert_eq!(details.arbeitgeber, None);
        assert_eq!(details.arbeitszeitmodelle.len(), 0);
    }

    #[test]
    fn test_address_deserialization() {
        let json = r#"{
            "land": "Deutschland",
            "region": "Bayern",
            "plz": "80331",
            "ort": "München",
            "strasse": "Hauptstraße",
            "strasseHausnummer": "Hauptstraße 123"
        }"#;

        let address: Address = serde_json::from_str(json).unwrap();
        assert_eq!(address.land, "Deutschland");
        assert_eq!(address.region, "Bayern");
        assert_eq!(address.ort, "München");
    }

    #[test]
    fn test_skill_deserialization() {
        let json = r#"{
            "hierarchieName": "Technical Skills",
            "auspraegungen": {
                "programming": ["Rust", "Python"],
                "tools": ["Git", "Docker"]
            }
        }"#;

        let skill: Skill = serde_json::from_str(json).unwrap();
        assert_eq!(skill.hierarchie_name, "Technical Skills");
        assert!(skill.auspraegungen.is_some());

        let auspraegungen = skill.auspraegungen.unwrap();
        assert!(auspraegungen.contains_key("programming"));
        assert!(auspraegungen.contains_key("tools"));
    }

    #[test]
    fn test_mobility_deserialization() {
        let json = r#"{
            "reisebereitschaft": "gelegentlich"
        }"#;

        let mobility: Mobility = serde_json::from_str(json).unwrap();
        assert_eq!(mobility.reisebereitschaft, Some("gelegentlich".to_string()));
    }

    #[test]
    fn test_leadership_skills_deserialization() {
        let json = r#"{
            "hatVollmacht": true,
            "hatBudgetverantwortung": false
        }"#;

        let skills: LeadershipSkills = serde_json::from_str(json).unwrap();
        assert_eq!(skills.hat_vollmacht, Some(true));
        assert_eq!(skills.hat_budgetverantwortung, Some(false));
    }

    #[test]
    fn test_coordinates_deserialization() {
        let json = r#"{
            "lat": 48.1351,
            "lon": 11.5820
        }"#;

        let coords: Coordinates = serde_json::from_str(json).unwrap();
        assert_eq!(coords.lat, 48.1351);
        assert_eq!(coords.lon, 11.5820);
    }

    #[test]
    fn test_angebotsart_equality() {
        assert_eq!(Angebotsart::Arbeit, Angebotsart::Arbeit);
        assert_ne!(Angebotsart::Arbeit, Angebotsart::Ausbildung);
    }

    #[test]
    fn test_befristung_equality() {
        assert_eq!(Befristung::Befristet, Befristung::Befristet);
        assert_ne!(Befristung::Befristet, Befristung::Unbefristet);
    }

    #[test]
    fn test_arbeitszeit_equality() {
        assert_eq!(Arbeitszeit::Vollzeit, Arbeitszeit::Vollzeit);
        assert_ne!(Arbeitszeit::Vollzeit, Arbeitszeit::Teilzeit);
    }

    #[test]
    fn test_job_listing_serialization() {
        let listing = JobListing {
            hash_id: Some("hash123".to_string()),
            refnr: "10001-TEST-S".to_string(),
            beruf: "Developer".to_string(),
            titel: Some("Senior Developer".to_string()),
            arbeitgeber: "Company".to_string(),
            aktuelle_veroeffentlichungsdatum: Some("2025-10-21".to_string()),
            eintrittsdatum: None,
            arbeitsort: WorkLocation {
                plz: Some("10115".to_string()),
                ort: Some("Berlin".to_string()),
                strasse: None,
                region: None,
                land: None,
                koordinaten: None,
                entfernung: None,
            },
            modifikations_timestamp: None,
            externe_url: None,
            kundennummer_hash: None,
        };

        let json = serde_json::to_string(&listing).unwrap();
        assert!(json.contains("10001-TEST-S"));
        assert!(json.contains("Developer"));
    }

    #[test]
    fn test_empty_job_search_response() {
        let json = r#"{
            "stellenangebote": []
        }"#;

        let response: JobSearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.stellenangebote.len(), 0);
        assert_eq!(response.max_ergebnisse, None);
    }
}
