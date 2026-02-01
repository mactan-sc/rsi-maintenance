use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;
use std::fs;
use std::path::Path;

pub struct I18n {
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    pub fn new(lang: LanguageIdentifier) -> Self {
        let tag = lang.to_string();
        let mut bundle = FluentBundle::new(vec![lang.clone()]);
        bundle.set_use_isolating(false);

        // Try to load a locale-specific FTL from assets/i18n at runtime.
        // Candidates include variants of the language identifier (e.g. "de-DE", "de_DE").
        let mut candidates = vec![tag.clone()];
        candidates.push(tag.replace('-', "_"));
        candidates.push(tag.to_lowercase());
        candidates.push(tag.replace('-', "_").to_lowercase());
        candidates.push(tag.to_uppercase());
        candidates.push(tag.replace('-', "_").to_uppercase());

        let mut ftl_content = None;
        for cand in candidates {
            let path = Path::new("assets").join("i18n").join(format!("{}.ftl", cand));
            if path.exists() {
                if let Ok(s) = fs::read_to_string(&path) {
                    ftl_content = Some(s);
                    break;
                }
            }
        }

        let ftl = match ftl_content {
            Some(s) => s,
            None => include_str!("../../assets/i18n/en-US.ftl").to_string(),
        };

        let resource = FluentResource::try_new(ftl).expect("Invalid FTL");

        bundle
            .add_resource(resource)
            .expect("Failed to add FTL resource");

        Self { bundle }
    }

    pub fn t(&self, key: &str) -> String {
        let msg = self
            .bundle
            .get_message(key)
            .unwrap_or_else(|| panic!("Missing key: {key}"));

        let pattern = msg.value().expect("Message has no value");

        self.bundle
            .format_pattern(pattern, None, &mut vec![])
            .to_string()
    }

    pub fn t_args(&self, key: &str, args: FluentArgs) -> String {
        let msg = self
            .bundle
            .get_message(key)
            .unwrap_or_else(|| panic!("Missing key: {key}"));

        let pattern = msg.value().expect("Message has no value");

        self.bundle
            .format_pattern(pattern, Some(&args), &mut vec![])
            .to_string()
    }
}