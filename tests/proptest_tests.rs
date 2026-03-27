use proptest::prelude::*;

use jobsuche::{
    decode_refnr, encode_refnr, JobListing, JobSearchResponse, SearchOptions, WorkLocation,
};

/// Strategy to generate an arbitrary WorkLocation with optional fields.
fn arb_work_location() -> impl Strategy<Value = WorkLocation> {
    (
        proptest::option::of("[a-zA-Z0-9 ]{0,20}"),
        proptest::option::of("[a-zA-Z ]{0,30}"),
        proptest::option::of("[a-zA-Z0-9 ]{0,50}"),
        proptest::option::of("[a-zA-Z ]{0,20}"),
        proptest::option::of("[a-zA-Z ]{0,20}"),
    )
        .prop_map(|(plz, ort, strasse, region, land)| WorkLocation {
            plz,
            ort,
            strasse,
            region,
            land,
            koordinaten: None,
            entfernung: None,
        })
}

/// Strategy to generate an arbitrary JobListing with required and optional fields.
fn arb_job_listing() -> impl Strategy<Value = JobListing> {
    (
        "[a-zA-Z0-9-]{5,20}",                    // refnr
        proptest::option::of("[a-zA-Z ]{3,30}"), // beruf
        "[a-zA-Z ]{3,30}",                       // arbeitgeber
        arb_work_location(),
        proptest::option::of("[a-zA-Z ]{3,40}"), // titel
    )
        .prop_map(
            |(refnr, beruf, arbeitgeber, arbeitsort, titel)| JobListing {
                hash_id: None,
                refnr,
                beruf,
                titel,
                arbeitgeber,
                aktuelle_veroeffentlichungsdatum: None,
                eintrittsdatum: None,
                arbeitsort,
                modifikations_timestamp: None,
                externe_url: None,
                kundennummer_hash: None,
            },
        )
}

/// Strategy to generate an arbitrary JobSearchResponse.
fn arb_job_search_response() -> impl Strategy<Value = JobSearchResponse> {
    (
        proptest::collection::vec(arb_job_listing(), 0..5),
        proptest::option::of(0u64..10000),
        proptest::option::of(0u64..100),
        proptest::option::of(1u64..100),
    )
        .prop_map(
            |(stellenangebote, max_ergebnisse, page, size)| JobSearchResponse {
                stellenangebote,
                max_ergebnisse,
                page,
                size,
                facetten: None,
            },
        )
}

proptest! {
    /// Any combination of builder methods produces valid URL-encoded query strings
    /// (no panics, valid UTF-8).
    #[test]
    fn search_options_builder_roundtrip(
        was in proptest::option::of(".*"),
        wo in proptest::option::of(".*"),
        arbeitgeber in proptest::option::of(".*"),
        berufsfeld in proptest::option::of(".*"),
        page in proptest::option::of(any::<u64>()),
        size in proptest::option::of(any::<u64>()),
        umkreis in proptest::option::of(any::<u64>()),
        veroeffentlichtseit in proptest::option::of(any::<u64>()),
        zeitarbeit in proptest::option::of(any::<bool>()),
        behinderung in proptest::option::of(any::<bool>()),
        corona in proptest::option::of(any::<bool>()),
    ) {
        let mut builder = SearchOptions::builder();

        let mut has_any = false;
        if let Some(ref v) = was { builder.was(v); has_any = true; }
        if let Some(ref v) = wo { builder.wo(v); has_any = true; }
        if let Some(ref v) = arbeitgeber { builder.arbeitgeber(v); has_any = true; }
        if let Some(ref v) = berufsfeld { builder.berufsfeld(v); has_any = true; }
        if let Some(v) = page { builder.page(v); has_any = true; }
        if let Some(v) = size { builder.size(v); has_any = true; }
        if let Some(v) = umkreis { builder.umkreis(v); has_any = true; }
        if let Some(v) = veroeffentlichtseit { builder.veroeffentlichtseit(v); has_any = true; }
        if let Some(v) = zeitarbeit { builder.zeitarbeit(v); has_any = true; }
        if let Some(v) = behinderung { builder.behinderung(v); has_any = true; }
        if let Some(v) = corona { builder.corona(v); has_any = true; }

        let options = builder.build();
        let serialized = options.serialize();

        if has_any {
            let query = serialized.unwrap();
            // The result must be valid UTF-8 (it is a String, so this is guaranteed).
            // Additionally, verify it is non-empty.
            prop_assert!(!query.is_empty());
            // Verify the string is valid UTF-8 by attempting to parse bytes.
            prop_assert!(std::str::from_utf8(query.as_bytes()).is_ok());
        }
    }

    /// For any u64 `s`, `size()` is always `Some(min(s, 100))`.
    #[test]
    fn size_capping_property(s in any::<u64>()) {
        let options = SearchOptions::builder().size(s).build();
        let expected = s.min(100);
        prop_assert_eq!(options.size(), Some(expected));
    }

    /// For any u64 `d`, the serialized veroeffentlichtseit value is always `min(d, 100)`.
    #[test]
    fn veroeffentlichtseit_capping_property(d in any::<u64>()) {
        let options = SearchOptions::builder().veroeffentlichtseit(d).build();
        let query = options.serialize().unwrap();
        let expected = d.min(100);
        let needle = format!("veroeffentlichtseit={}", expected);
        prop_assert!(
            query.contains(&needle),
            "Expected query to contain '{}', got '{}'", needle, query
        );
    }

    /// For any non-empty ASCII string, `decode_refnr(encode_refnr(s))` returns the original.
    #[test]
    fn base64_encode_decode_roundtrip(s in "[[:ascii:]]{1,100}") {
        let encoded = encode_refnr(&s);
        let decoded = decode_refnr(&encoded).unwrap();
        prop_assert_eq!(&decoded, &s);
    }

    /// Serde roundtrip for JobSearchResponse: serialize to JSON and deserialize back,
    /// verify fields match.
    #[test]
    fn serde_roundtrip_job_search_response(response in arb_job_search_response()) {
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: JobSearchResponse = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(deserialized.stellenangebote.len(), response.stellenangebote.len());
        prop_assert_eq!(deserialized.max_ergebnisse, response.max_ergebnisse);
        prop_assert_eq!(deserialized.page, response.page);
        prop_assert_eq!(deserialized.size, response.size);

        for (orig, deser) in response.stellenangebote.iter().zip(deserialized.stellenangebote.iter()) {
            prop_assert_eq!(&deser.refnr, &orig.refnr);
            prop_assert_eq!(&deser.beruf, &orig.beruf);
            prop_assert_eq!(&deser.arbeitgeber, &orig.arbeitgeber);
            prop_assert_eq!(&deser.arbeitsort.ort, &orig.arbeitsort.ort);
            prop_assert_eq!(&deser.arbeitsort.plz, &orig.arbeitsort.plz);
            prop_assert_eq!(&deser.arbeitsort.region, &orig.arbeitsort.region);
            prop_assert_eq!(&deser.arbeitsort.land, &orig.arbeitsort.land);
        }
    }

    /// Serde roundtrip for WorkLocation: generate random WorkLocation with optional fields,
    /// roundtrip through JSON.
    #[test]
    fn serde_roundtrip_work_location(location in arb_work_location()) {
        let json = serde_json::to_string(&location).unwrap();
        let deserialized: WorkLocation = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(&deserialized.plz, &location.plz);
        prop_assert_eq!(&deserialized.ort, &location.ort);
        prop_assert_eq!(&deserialized.strasse, &location.strasse);
        prop_assert_eq!(&deserialized.region, &location.region);
        prop_assert_eq!(&deserialized.land, &location.land);
    }
}
