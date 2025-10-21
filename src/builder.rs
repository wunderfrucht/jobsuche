//! Builder pattern for search options

use std::collections::BTreeMap;
use url::form_urlencoded;

use crate::rep::{Angebotsart, Arbeitszeit, Befristung};

/// Options available for job search
#[derive(Default, Clone, Debug)]
pub struct SearchOptions {
    params: BTreeMap<&'static str, String>,
}

impl SearchOptions {
    /// Return a new instance of a builder for options
    pub fn builder() -> SearchOptionsBuilder {
        SearchOptionsBuilder::new()
    }

    /// Serialize options as a query string. Returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            Some(
                form_urlencoded::Serializer::new(String::new())
                    .extend_pairs(&self.params)
                    .finish(),
            )
        }
    }

    /// Convert back to a builder for modification
    pub fn as_builder(&self) -> SearchOptionsBuilder {
        SearchOptionsBuilder::copy_from(self)
    }

    /// Get the page value from search options
    pub fn page(&self) -> Option<u64> {
        self.params.get("page").and_then(|s| s.parse().ok())
    }

    /// Get the size value from search options
    pub fn size(&self) -> Option<u64> {
        self.params.get("size").and_then(|s| s.parse().ok())
    }
}

/// A builder interface for search options. Typically this is initialized with SearchOptions::builder()
#[derive(Default, Debug)]
pub struct SearchOptionsBuilder {
    params: BTreeMap<&'static str, String>,
}

impl SearchOptionsBuilder {
    /// Create a new SearchOptionsBuilder
    pub fn new() -> SearchOptionsBuilder {
        SearchOptionsBuilder {
            ..Default::default()
        }
    }

    fn copy_from(search_options: &SearchOptions) -> SearchOptionsBuilder {
        SearchOptionsBuilder {
            params: search_options.params.clone(),
        }
    }

    /// Free text search for job title
    ///
    /// # Example
    /// ```
    /// use jobsuche::SearchOptions;
    ///
    /// let options = SearchOptions::builder()
    ///     .was("Softwareentwickler")
    ///     .build();
    /// ```
    pub fn was(&mut self, job_title: &str) -> &mut SearchOptionsBuilder {
        self.params.insert("was", job_title.to_string());
        self
    }

    /// Free text search for location
    ///
    /// # Example
    /// ```
    /// use jobsuche::SearchOptions;
    ///
    /// let options = SearchOptions::builder()
    ///     .wo("Berlin")
    ///     .build();
    /// ```
    pub fn wo(&mut self, location: &str) -> &mut SearchOptionsBuilder {
        self.params.insert("wo", location.to_string());
        self
    }

    /// Free text search for occupational field
    ///
    /// # Example
    /// ```
    /// use jobsuche::SearchOptions;
    ///
    /// let options = SearchOptions::builder()
    ///     .berufsfeld("Informatik")
    ///     .build();
    /// ```
    pub fn berufsfeld(&mut self, field: &str) -> &mut SearchOptionsBuilder {
        self.params.insert("berufsfeld", field.to_string());
        self
    }

    /// Page number for pagination (starting from 1)
    ///
    /// # Example
    /// ```
    /// use jobsuche::SearchOptions;
    ///
    /// let options = SearchOptions::builder()
    ///     .page(1)
    ///     .build();
    /// ```
    pub fn page(&mut self, p: u64) -> &mut SearchOptionsBuilder {
        self.params.insert("page", p.to_string());
        self
    }

    /// Number of results per page (max 100)
    ///
    /// # Example
    /// ```
    /// use jobsuche::SearchOptions;
    ///
    /// let options = SearchOptions::builder()
    ///     .size(50)
    ///     .build();
    /// ```
    pub fn size(&mut self, s: u64) -> &mut SearchOptionsBuilder {
        let capped = s.min(100); // API limit is 100
        self.params.insert("size", capped.to_string());
        self
    }

    /// Filter by employer name (exact match, case-sensitive)
    ///
    /// Note: According to Issue #52, employer search is case-sensitive and exact-match only.
    /// "Deutsche Bahn AG" works, but "deutsche bahn" or "bahn" won't.
    ///
    /// # Example
    /// ```
    /// use jobsuche::SearchOptions;
    ///
    /// let options = SearchOptions::builder()
    ///     .arbeitgeber("Deutsche Bahn AG")
    ///     .build();
    /// ```
    pub fn arbeitgeber(&mut self, employer: &str) -> &mut SearchOptionsBuilder {
        self.params.insert("arbeitgeber", employer.to_string());
        self
    }

    /// Filter by days since publication (0-100 days)
    ///
    /// # Example
    /// ```
    /// use jobsuche::SearchOptions;
    ///
    /// let options = SearchOptions::builder()
    ///     .veroeffentlichtseit(7) // Jobs from last 7 days
    ///     .build();
    /// ```
    pub fn veroeffentlichtseit(&mut self, days: u64) -> &mut SearchOptionsBuilder {
        let capped = days.min(100); // API limit is 100
        self.params
            .insert("veroeffentlichtseit", capped.to_string());
        self
    }

    /// Include or exclude temporary employment agencies (default: true)
    ///
    /// # Example
    /// ```
    /// use jobsuche::SearchOptions;
    ///
    /// let options = SearchOptions::builder()
    ///     .zeitarbeit(false) // Exclude temp agencies
    ///     .build();
    /// ```
    pub fn zeitarbeit(&mut self, include: bool) -> &mut SearchOptionsBuilder {
        self.params.insert("zeitarbeit", include.to_string());
        self
    }

    /// Filter by employment type
    ///
    /// # Example
    /// ```
    /// use jobsuche::{SearchOptions, Angebotsart};
    ///
    /// let options = SearchOptions::builder()
    ///     .angebotsart(Angebotsart::Arbeit)
    ///     .build();
    /// ```
    pub fn angebotsart(&mut self, art: Angebotsart) -> &mut SearchOptionsBuilder {
        self.params.insert("angebotsart", art.as_str().to_string());
        self
    }

    /// Filter by contract type (can specify multiple, semicolon-separated)
    ///
    /// # Example
    /// ```
    /// use jobsuche::{SearchOptions, Befristung};
    ///
    /// let options = SearchOptions::builder()
    ///     .befristung(vec![Befristung::Unbefristet])
    ///     .build();
    /// ```
    pub fn befristung(&mut self, types: Vec<Befristung>) -> &mut SearchOptionsBuilder {
        let value = types
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(";");
        self.params.insert("befristung", value);
        self
    }

    /// Filter by working time model (can specify multiple, semicolon-separated)
    ///
    /// # Example
    /// ```
    /// use jobsuche::{SearchOptions, Arbeitszeit};
    ///
    /// let options = SearchOptions::builder()
    ///     .arbeitszeit(vec![Arbeitszeit::Vollzeit, Arbeitszeit::Teilzeit])
    ///     .build();
    /// ```
    pub fn arbeitszeit(&mut self, times: Vec<Arbeitszeit>) -> &mut SearchOptionsBuilder {
        let value = times
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(";");
        self.params.insert("arbeitszeit", value);
        self
    }

    /// Filter for jobs suitable for people with disabilities
    ///
    /// # Example
    /// ```
    /// use jobsuche::SearchOptions;
    ///
    /// let options = SearchOptions::builder()
    ///     .behinderung(true)
    ///     .build();
    /// ```
    pub fn behinderung(&mut self, suitable: bool) -> &mut SearchOptionsBuilder {
        self.params.insert("behinderung", suitable.to_string());
        self
    }

    /// Filter for jobs offered in the context of Corona/COVID-19
    ///
    /// # Example
    /// ```
    /// use jobsuche::SearchOptions;
    ///
    /// let options = SearchOptions::builder()
    ///     .corona(true)
    ///     .build();
    /// ```
    pub fn corona(&mut self, corona_related: bool) -> &mut SearchOptionsBuilder {
        self.params.insert("corona", corona_related.to_string());
        self
    }

    /// Search radius in kilometers from the location (wo parameter)
    ///
    /// # Example
    /// ```
    /// use jobsuche::SearchOptions;
    ///
    /// let options = SearchOptions::builder()
    ///     .wo("Frankfurt")
    ///     .umkreis(50) // 50km radius
    ///     .build();
    /// ```
    pub fn umkreis(&mut self, radius_km: u64) -> &mut SearchOptionsBuilder {
        self.params.insert("umkreis", radius_km.to_string());
        self
    }

    /// Build the final SearchOptions
    pub fn build(&self) -> SearchOptions {
        SearchOptions {
            params: self.params.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let options = SearchOptions::builder()
            .was("Softwareentwickler")
            .wo("Berlin")
            .page(1)
            .size(25)
            .build();

        let query = options.serialize().unwrap();
        assert!(query.contains("was=Softwareentwickler"));
        assert!(query.contains("wo=Berlin"));
        assert!(query.contains("page=1"));
        assert!(query.contains("size=25"));
    }

    #[test]
    fn test_builder_arbeitszeit() {
        let options = SearchOptions::builder()
            .arbeitszeit(vec![Arbeitszeit::Vollzeit, Arbeitszeit::Teilzeit])
            .build();

        let query = options.serialize().unwrap();
        assert!(query.contains("arbeitszeit=vz%3Btz"));
    }

    #[test]
    fn test_size_capping() {
        let options = SearchOptions::builder()
            .size(200) // Should be capped at 100
            .build();

        assert_eq!(options.size(), Some(100));
    }

    #[test]
    fn test_berufsfeld() {
        let options = SearchOptions::builder().berufsfeld("Informatik").build();

        let query = options.serialize().unwrap();
        assert!(query.contains("berufsfeld=Informatik"));
    }

    #[test]
    fn test_arbeitgeber() {
        let options = SearchOptions::builder()
            .arbeitgeber("Deutsche Bahn AG")
            .build();

        let query = options.serialize().unwrap();
        assert!(query.contains("arbeitgeber=Deutsche+Bahn+AG"));
    }

    #[test]
    fn test_veroeffentlichtseit() {
        let options = SearchOptions::builder().veroeffentlichtseit(7).build();

        let query = options.serialize().unwrap();
        assert!(query.contains("veroeffentlichtseit=7"));
    }

    #[test]
    fn test_veroeffentlichtseit_capping() {
        let options = SearchOptions::builder()
            .veroeffentlichtseit(150) // Should be capped at 100
            .build();

        let query = options.serialize().unwrap();
        assert!(query.contains("veroeffentlichtseit=100"));
    }

    #[test]
    fn test_zeitarbeit_true() {
        let options = SearchOptions::builder().zeitarbeit(true).build();

        let query = options.serialize().unwrap();
        assert!(query.contains("zeitarbeit=true"));
    }

    #[test]
    fn test_zeitarbeit_false() {
        let options = SearchOptions::builder().zeitarbeit(false).build();

        let query = options.serialize().unwrap();
        assert!(query.contains("zeitarbeit=false"));
    }

    #[test]
    fn test_angebotsart() {
        let options = SearchOptions::builder()
            .angebotsart(Angebotsart::Arbeit)
            .build();

        let query = options.serialize().unwrap();
        assert!(query.contains("angebotsart=1"));
    }

    #[test]
    fn test_angebotsart_ausbildung() {
        let options = SearchOptions::builder()
            .angebotsart(Angebotsart::Ausbildung)
            .build();

        let query = options.serialize().unwrap();
        assert!(query.contains("angebotsart=4"));
    }

    #[test]
    fn test_befristung_single() {
        let options = SearchOptions::builder()
            .befristung(vec![Befristung::Unbefristet])
            .build();

        let query = options.serialize().unwrap();
        assert!(query.contains("befristung=2"));
    }

    #[test]
    fn test_befristung_multiple() {
        let options = SearchOptions::builder()
            .befristung(vec![Befristung::Befristet, Befristung::Unbefristet])
            .build();

        let query = options.serialize().unwrap();
        assert!(query.contains("befristung=1%3B2"));
    }

    #[test]
    fn test_behinderung() {
        let options = SearchOptions::builder().behinderung(true).build();

        let query = options.serialize().unwrap();
        assert!(query.contains("behinderung=true"));
    }

    #[test]
    fn test_corona() {
        let options = SearchOptions::builder().corona(true).build();

        let query = options.serialize().unwrap();
        assert!(query.contains("corona=true"));
    }

    #[test]
    fn test_umkreis() {
        let options = SearchOptions::builder().wo("Frankfurt").umkreis(50).build();

        let query = options.serialize().unwrap();
        assert!(query.contains("umkreis=50"));
    }

    #[test]
    fn test_as_builder() {
        let original = SearchOptions::builder()
            .was("Developer")
            .wo("Berlin")
            .size(10)
            .build();

        let modified = original
            .as_builder()
            .page(2) // Add page to existing options
            .build();

        let query = modified.serialize().unwrap();
        assert!(query.contains("was=Developer"));
        assert!(query.contains("wo=Berlin"));
        assert!(query.contains("size=10"));
        assert!(query.contains("page=2"));
    }

    #[test]
    fn test_page_getter() {
        let options = SearchOptions::builder().page(5).build();
        assert_eq!(options.page(), Some(5));
    }

    #[test]
    fn test_size_getter() {
        let options = SearchOptions::builder().size(25).build();
        assert_eq!(options.size(), Some(25));
    }

    #[test]
    fn test_page_getter_none() {
        let options = SearchOptions::builder().was("test").build();
        assert_eq!(options.page(), None);
    }

    #[test]
    fn test_size_getter_none() {
        let options = SearchOptions::builder().was("test").build();
        assert_eq!(options.size(), None);
    }

    #[test]
    fn test_serialize_empty() {
        let options = SearchOptions::builder().build();
        assert_eq!(options.serialize(), None);
    }

    #[test]
    fn test_default_search_options() {
        let options = SearchOptions::default();
        assert_eq!(options.serialize(), None);
    }

    #[test]
    fn test_combined_filters() {
        let options = SearchOptions::builder()
            .was("Software Engineer")
            .wo("Munich")
            .arbeitszeit(vec![Arbeitszeit::Vollzeit])
            .befristung(vec![Befristung::Unbefristet])
            .angebotsart(Angebotsart::Arbeit)
            .umkreis(25)
            .size(50)
            .page(1)
            .build();

        let query = options.serialize().unwrap();
        assert!(query.contains("was=Software+Engineer"));
        assert!(query.contains("wo=Munich"));
        assert!(query.contains("arbeitszeit=vz"));
        assert!(query.contains("befristung=2"));
        assert!(query.contains("angebotsart=1"));
        assert!(query.contains("umkreis=25"));
        assert!(query.contains("size=50"));
        assert!(query.contains("page=1"));
    }

    #[test]
    fn test_multiple_arbeitszeit() {
        let options = SearchOptions::builder()
            .arbeitszeit(vec![
                Arbeitszeit::Vollzeit,
                Arbeitszeit::Teilzeit,
                Arbeitszeit::HeimTelearbeit,
            ])
            .build();

        let query = options.serialize().unwrap();
        assert!(query.contains("arbeitszeit=vz%3Btz%3Bho"));
    }

    #[test]
    fn test_arbeitszeit_minijob() {
        let options = SearchOptions::builder()
            .arbeitszeit(vec![Arbeitszeit::Minijob])
            .build();

        let query = options.serialize().unwrap();
        assert!(query.contains("arbeitszeit=mj"));
    }

    #[test]
    fn test_arbeitszeit_schicht() {
        let options = SearchOptions::builder()
            .arbeitszeit(vec![Arbeitszeit::SchichtNachtarbeitWochenende])
            .build();

        let query = options.serialize().unwrap();
        assert!(query.contains("arbeitszeit=snw"));
    }
}
