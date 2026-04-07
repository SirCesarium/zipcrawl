use crate::archive::{ZipEntry, ZipManager};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::cmp::Reverse;

pub enum InputMode {
    Normal,
    Search,
}

pub struct App<'a> {
    pub manager: &'a mut ZipManager,
    pub all_entries: Vec<ZipEntry>,
    pub current_entries: Vec<ZipEntry>,
    pub current_path: String,
    pub selected_index: usize,
    pub preview_content: Option<String>,
    pub show_preview: bool,
    pub preview_scroll: u16,
    pub search_query: String,
    pub input_mode: InputMode,
}

impl<'a> App<'a> {
    pub fn new(manager: &'a mut ZipManager) -> Self {
        let entries = manager.entries().unwrap_or_default();
        let mut app = Self {
            manager,
            all_entries: entries,
            current_entries: Vec::new(),
            current_path: String::new(),
            selected_index: 0,
            preview_content: None,
            show_preview: false,
            preview_scroll: 0,
            search_query: String::new(),
            input_mode: InputMode::Normal,
        };
        app.apply_filter();
        app
    }

    pub fn apply_filter(&mut self) {
        let matcher = SkimMatcherV2::default();
        let current = &self.current_path;

        self.current_entries = self
            .all_entries
            .iter()
            .filter(|e| {
                if !self.search_query.is_empty() {
                    return matcher.fuzzy_match(&e.name, &self.search_query).is_some();
                }

                if !e.name.starts_with(current) || &e.name == current {
                    return false;
                }

                let relative = &e.name[current.len()..];
                relative.split('/').filter(|s| !s.is_empty()).count() == 1
            })
            .cloned()
            .collect();

        if !self.search_query.is_empty() {
            self.current_entries.sort_by_cached_key(|e| {
                Reverse(
                    matcher
                        .fuzzy_match(&e.name, &self.search_query)
                        .unwrap_or(0),
                )
            });
        }

        self.selected_index = self
            .selected_index
            .min(self.current_entries.len().saturating_sub(1));
    }

    pub fn enter_directory(&mut self) {
        if let Some(entry) = self.current_entries.get(self.selected_index) {
            if entry.is_dir {
                self.current_path = entry.name.clone();
                self.search_query.clear();
                self.apply_filter();
            } else {
                self.load_preview();
                self.show_preview = true;
            }
        }
    }

    pub fn go_back(&mut self) {
        if self.current_path.is_empty() {
            return;
        }
        let mut parts: Vec<&str> = self
            .current_path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        parts.pop();
        self.current_path = if parts.is_empty() {
            String::new()
        } else {
            let mut p = parts.join("/");
            p.push('/');
            p
        };
        self.search_query.clear();
        self.apply_filter();
    }

    pub fn load_preview(&mut self) {
        if let Some(entry) = self.current_entries.get(self.selected_index)
            && !entry.is_dir
            && let Ok(content) = self.manager.read_file_content(&entry.name)
        {
            self.preview_content = Some(String::from_utf8_lossy(&content).into_owned());
            self.preview_scroll = 0;
            return;
        }
        self.preview_content = None;
    }
}
