use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::app::App;

pub fn search_apps<'a>(apps: &'a [App], query: &str) -> Vec<&'a App> {
    let matcher = SkimMatcherV2::default();

    let mut results: Vec<(&App, i64)> = apps
        .iter()
        .filter_map(|app| {
            let score = matcher.fuzzy_match(&app.name, query)?;
            Some((app, score))
        })
        .collect();

    results.sort_by(|a, b| b.1.cmp(&a.1));
    results.into_iter().map(|(app, _)| app).collect()
}
