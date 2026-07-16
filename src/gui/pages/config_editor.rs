use crate::utility::*;
use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Space},
    Alignment, Element, Length,
};
use std::sync::OnceLock;

pub struct ConfigEditorPage {
    pub title: &'static str,
    pub add_entry: &'static str,
    pub back: &'static str,
}

impl Default for ConfigEditorPage {
    fn default() -> Self {
        let lang = detect_lang();
        let i18n = I18n::new(lang);

        Self {
            title: Box::leak(i18n.t("Config-Editor").into_boxed_str()),
            add_entry: Box::leak(i18n.t("Add-Entry").into_boxed_str()),
            back: Box::leak(i18n.t("Back").into_boxed_str()),
        }
    }
}

// One row of the key-value editor
#[derive(Debug, Clone)]
pub struct Entry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    AddEntry,
    RemoveEntry(usize),
    EditKey(usize, String),
    EditValue(usize, String),
}

// The runtime state for the config editor screen
pub struct State {
    pub entries: Vec<Entry>,
    pub original_config: ConfigFile,
}

impl State {
    pub fn new() -> Self {
        let config = load_full_config();
        let entries: Vec<Entry> = config
            .environment
            .iter()
            .map(|(k, v)| Entry {
                key: k.clone(),
                value: v.clone(),
            })
            .collect();
        Self {
            entries,
            original_config: config,
        }
    }

    pub fn save(&self) {
        let mut config = self.original_config.clone();
        config.environment.clear();
        for entry in &self.entries {
            if !entry.key.trim().is_empty() {
                config
                    .environment
                    .insert(entry.key.trim().to_string(), entry.value.clone());
            }
        }
        save_full_config(&config);
    }
}

pub fn view<'a>(state: &'a State) -> Element<'a, Message> {
    static PAGE: OnceLock<ConfigEditorPage> = OnceLock::new();
    let page = PAGE.get_or_init(ConfigEditorPage::default);

    let header = row![
        text("Key").width(Length::Fill),
        text("Value").width(Length::Fill),
        // Spacer for the delete button column.
        Space::new().width(Length::Fixed(36.0)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let mut rows = column![header].spacing(4);

    for (i, entry) in state.entries.iter().enumerate() {
        let key_input = text_input("Key", &entry.key)
            .on_input(move |s| Message::EditKey(i, s))
            .width(Length::Fill);

        let val_input = text_input("Value", &entry.value)
            .on_input(move |s| Message::EditValue(i, s))
            .width(Length::Fill);

        let del_btn = button(text("✕").size(14))
            .on_press(Message::RemoveEntry(i))
            .padding(4);

        let entry_row = row![key_input, val_input, del_btn]
            .spacing(8)
            .align_y(Alignment::Center);

        rows = rows.push(entry_row);
    }

    let add_btn =
        button(text(page.add_entry)).on_press(Message::AddEntry);

    let actions = row![
        add_btn,
        Space::new().width(Length::Fill),
        button(text(page.back)).on_press(Message::Back),
    ]
    .spacing(8);

    let content = column![
        text(page.title).size(24),
        scrollable(rows).height(Length::Fill),
        actions,
    ]
    .spacing(12);

    container(content)
        .center_x(Length::Fill)
        .padding(24)
        .into()
}
