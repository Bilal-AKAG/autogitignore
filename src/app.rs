use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    Confirm,
}

#[derive(Debug, PartialEq)]
pub enum PreviewMode {
    Highlighted,
    Combined,
}

#[derive(Debug, PartialEq)]
pub enum ConfirmAction {
    Append,
    Overwrite,
}

/// Application state and business logic.
pub struct App {
    /// List of all available template names.
    pub templates: Vec<String>,
    /// List of template names that match the current search query.
    pub filtered_templates: Vec<String>,
    /// Set of selected template names.
    pub selected_templates: HashSet<String>,
    /// Current index in the filtered templates list.
    pub highlighted_index: usize,
    /// Current search input string.
    pub search_query: String,
    /// Current input mode (Normal, Editing, or Confirm).
    pub input_mode: InputMode,
    /// Mapping of template names to their actual .gitignore content.
    pub template_contents: HashMap<String, String>,
    /// Whether the application is still fetching data.
    pub is_loading: bool,
    /// Current error message to display in the UI.
    pub error: Option<String>,
    /// Current success/info notification to display in the UI.
    pub notification: Option<String>,
    /// Scroll offset for the preview pane.
    pub preview_scroll: u16,
    /// Fuzzy matcher for filtering templates.
    pub matcher: SkimMatcherV2,
    /// Current preview view mode.
    pub preview_mode: PreviewMode,
    /// Currently selected action in the confirmation modal.
    pub confirm_action: Option<ConfirmAction>,
    /// Whether the app should exit after the next successful save.
    pub should_quit_after_save: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
            filtered_templates: Vec::new(),
            selected_templates: HashSet::new(),
            highlighted_index: 0,
            search_query: String::new(),
            input_mode: InputMode::Editing,
            template_contents: HashMap::new(),
            is_loading: true,
            error: None,
            notification: None,
            preview_scroll: 0,
            matcher: SkimMatcherV2::default(),
            preview_mode: PreviewMode::Highlighted,
            confirm_action: None,
            should_quit_after_save: false,
        }
    }

    pub fn set_templates(&mut self, templates: Vec<String>) {
        self.templates = templates;
        self.templates.sort();
        self.apply_filter();
        self.is_loading = false;
    }

    pub fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_templates = self.templates.clone();
        } else {
            let mut matches: Vec<(i64, String)> = self
                .templates
                .iter()
                .filter_map(|t| {
                    self.matcher
                        .fuzzy_match(t, &self.search_query)
                        .map(|score| (score, t.clone()))
                })
                .collect();

            matches.sort_by(|a, b| b.0.cmp(&a.0));
            self.filtered_templates = matches.into_iter().map(|(_, t)| t).collect();
        }

        if self.highlighted_index >= self.filtered_templates.len()
            && !self.filtered_templates.is_empty()
        {
            self.highlighted_index = self.filtered_templates.len() - 1;
        } else if self.filtered_templates.is_empty() {
            self.highlighted_index = 0;
        }
    }

    pub fn next(&mut self) {
        if !self.filtered_templates.is_empty() {
            self.highlighted_index = (self.highlighted_index + 1) % self.filtered_templates.len();
            self.preview_scroll = 0;
        }
    }

    pub fn previous(&mut self) {
        if !self.filtered_templates.is_empty() {
            if self.highlighted_index > 0 {
                self.highlighted_index -= 1;
            } else {
                self.highlighted_index = self.filtered_templates.len() - 1;
            }
            self.preview_scroll = 0;
        }
    }

    /// Toggles selection of the currently highlighted template and clears any errors.
    pub fn toggle_selection(&mut self) {
        if let Some(template) = self.filtered_templates.get(self.highlighted_index) {
            if self.selected_templates.contains(template) {
                self.selected_templates.remove(template);
            } else {
                self.selected_templates.insert(template.clone());
            }
        }
        self.error = None;
        self.notification = None;
    }

    pub fn get_current_highlighted(&self) -> Option<String> {
        self.filtered_templates.get(self.highlighted_index).cloned()
    }

    pub fn get_combined_preview(&self) -> String {
        match self.preview_mode {
            PreviewMode::Highlighted => {
                if let Some(t) = self.get_current_highlighted() {
                    let content = self
                        .template_contents
                        .get(&t)
                        .cloned()
                        .unwrap_or_else(|| "Loading preview...".to_string());
                    format!("--- PREVIEWING: {} ---\n\n{}", t, content)
                } else {
                    "No template highlighted.".to_string()
                }
            }
            PreviewMode::Combined => {
                if self.selected_templates.is_empty() {
                    return "No templates selected. Use [Highlighted] view to see templates."
                        .to_string();
                }

                let mut combined = String::new();
                let mut sorted_selected: Vec<_> = self.selected_templates.iter().collect();
                sorted_selected.sort();

                for t in sorted_selected {
                    combined.push_str(&format!("### {} ###\n", t));
                    combined.push_str(
                        self.template_contents
                            .get(t)
                            .map(|s| s.as_str())
                            .unwrap_or("Loading..."),
                    );
                    combined.push_str("\n\n");
                }
                combined
            }
        }
    }

    pub fn get_preview_line_count(&self) -> usize {
        self.get_combined_preview().lines().count()
    }

    pub fn generate_gitignore_content(&self) -> String {
        let mut sorted_selected: Vec<_> = self.selected_templates.iter().collect();
        sorted_selected.sort();

        let mut combined = String::new();
        for t in sorted_selected {
            combined.push_str(&format!("\n# --- {} ---\n", t));
            combined.push_str(self.template_contents.get(t).map(|s| s.as_str()).unwrap_or(""));
            combined.push('\n');
        }
        combined
    }

    pub fn get_selected_names_summary(&self) -> String {
        let mut selected: Vec<_> = self.selected_templates.iter().collect();
        selected.sort();
        selected.into_iter().cloned().collect::<Vec<_>>().join(", ")
    }
}
