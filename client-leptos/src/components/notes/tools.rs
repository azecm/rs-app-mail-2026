use super::modals::{NOTES_MODAL, NotesModals};
use super::utils::{CurrentIdn, ReadNotesMap, position_max, row_by_idn, update_counts};
use crate::components::notes::modals::get_current_row;
use crate::store::APP_NOTES_EDITING;
use crate::utils::{mail_to, svg_icon};
use leptos::ev;
use leptos::html::*;
use leptos::prelude::*;

pub fn view_notes_tools() -> impl IntoView {
    let notes_map = use_context::<ReadNotesMap>().expect("context::notes_map");
    let current_idn = use_context::<CurrentIdn>().expect("context::current_idn");
    div()
        .class("notes-tools")
        .child(div().class("notes-tools__buttons").child(move || {
            action_list(notes_map, current_idn)
                .iter()
                .map(|row| button_view(row.0, row.1, row.2))
                .collect_view()
        }))
        .child("заметок: ")
        .child(update_counts)
}

fn button_view(icon: &str, label: &str, action: ToolAction) -> impl IntoView {
    button()
        .title(label)
        .child(svg_icon(icon))
        .on(ev::click, move |_| {
            actions(action);
        })
}

fn actions(action: ToolAction) {
    match action {
        ToolAction::AddEvent
        | ToolAction::EditEvent
        | ToolAction::AddItem
        | ToolAction::EditItem
        | ToolAction::RemoveItem
        | ToolAction::AddGroup
        | ToolAction::EditGroup
        | ToolAction::RemoveGroup => {
            NOTES_MODAL.set(tool_to_modal(action));
        }
        // --
        ToolAction::ContentCancel => {
            APP_NOTES_EDITING.set(None);
        }
        ToolAction::ContentEdit => {
            APP_NOTES_EDITING.set(Some(true));
        }
        ToolAction::ContentSave => {
            APP_NOTES_EDITING.set(Some(false));
        }
        // --
        ToolAction::WriteLetter => {
            write_letter();
        }
    }
}

fn write_letter() {
    let row = get_current_row();
    let label = row.label.unwrap_or_default();
    let email = row.email.unwrap_or_default();
    let recipient = if !label.is_empty() {
        format!("{label} <{email}>")
    } else {
        email
    };

    let (subject, content) = letter_data(row.content.unwrap_or_default());

    mail_to(&recipient, &subject, &content);
}

fn letter_data(source: String) -> (String, String) {
    let marker_subject = "subject:";
    let marker_content_start = "content:";
    let marker_content_end = ":content";

    let mut flag_content = false;

    let mut subject = "".to_string();
    let mut content: Vec<String> = vec![];

    for line in source.split("\n") {
        let text: String = line.trim().to_string();
        if let Some(text) = text.strip_prefix(marker_subject) {
            subject = text.trim().to_string();
        }
        if text.contains(marker_content_end) {
            flag_content = false;
        }
        if flag_content {
            content.push(text.to_string());
        }
        if text.contains(marker_content_start) {
            flag_content = true;
        }
    }

    (subject, content.join("\n"))
}

fn tool_to_modal(action: ToolAction) -> Option<NotesModals> {
    match action {
        ToolAction::AddEvent => Some(NotesModals::AddEvent),
        ToolAction::EditEvent => Some(NotesModals::EditEvent),
        // --
        ToolAction::AddItem => Some(NotesModals::AddItem),
        ToolAction::EditItem => Some(NotesModals::EditItem),
        ToolAction::RemoveItem => Some(NotesModals::RemoveItem),
        // --
        ToolAction::AddGroup => Some(NotesModals::AddGroup),
        ToolAction::EditGroup => Some(NotesModals::EditGroup),
        ToolAction::RemoveGroup => Some(NotesModals::RemoveGroup),
        // --
        _ => None,
    }
}

type ToolList = (&'static str, &'static str, ToolAction);

#[derive(Debug, Clone, Copy)]
enum ToolAction {
    ContentCancel,
    ContentSave,
    ContentEdit,
    // --
    EditItem,
    AddItem,
    RemoveItem,
    // --
    WriteLetter,
    // --
    EditEvent,
    //RemoveEvent,
    AddEvent,
    // --
    EditGroup,
    AddGroup,
    RemoveGroup,
}

fn action_list(map: ReadNotesMap, current: CurrentIdn) -> Vec<ToolList> {
    let mut list: Vec<ToolList> = vec![];

    let current_idn = current.get();
    let map = map.get();

    if let Some(row) = row_by_idn(current_idn) {
        if let Some(idp) = row.idp
            && idp > 0
        {
            let has_event = row.event.flatten().is_some();
            let has_email = !row.email.unwrap_or_default().is_empty();

            if APP_NOTES_EDITING.get() == Some(true) {
                list.push((BACK_ARROW_SVG, "назад", ToolAction::ContentCancel));
                list.push((EDIT_ITEM_SVG, "сохранить заметку", ToolAction::ContentSave));
            } else {
                list.push((
                    EDIT_PROPS_SVG,
                    "редактировать свойства",
                    ToolAction::EditItem,
                ));

                list.push((
                    EDIT_ITEM_SVG,
                    "редактировать заметку",
                    ToolAction::ContentEdit,
                ));

                if has_email {
                    list.push((WRITE_LETTER_SVG, "написать письмо", ToolAction::WriteLetter));
                }
                if has_event {
                    list.push((
                        EDIT_EVENT_SVG,
                        "редактировать событие",
                        ToolAction::EditEvent,
                    ));
                } else {
                    list.push((ADD_EVENT_SVG, "добавить событие", ToolAction::AddEvent));
                }
                list.push((REMOVE_SVG, "удалить заметку", ToolAction::RemoveItem));
            }
        } else {
            let is_empty = position_max(&map, row.idn) == 0;

            list.push((ADD_FOLDER_SVG, "создать группу", ToolAction::AddGroup));
            list.push((
                EDIT_PROPS_SVG,
                "редактировать свойства",
                ToolAction::EditGroup,
            ));
            list.push((ADD_ITEM_SVG, "создать заметку", ToolAction::AddItem));

            if is_empty {
                list.push((REMOVE_SVG, "удалить группу", ToolAction::RemoveGroup));
            }
        }
    } else {
        list.push((ADD_FOLDER_SVG, "создать группу", ToolAction::AddGroup));
    }

    list
}

const ADD_ITEM_SVG: &str = include_str!("../../../../assets/icons/notes/add-item.svg");
const ADD_FOLDER_SVG: &str = include_str!("../../../../assets/icons/notes/add-folder.svg");
const EDIT_PROPS_SVG: &str = include_str!("../../../../assets/icons/notes/edit-props.svg");
const EDIT_ITEM_SVG: &str = include_str!("../../../../assets/icons/notes/edit-item.svg");
const REMOVE_SVG: &str = include_str!("../../../../assets/icons/notes/remove.svg");
const WRITE_LETTER_SVG: &str = include_str!("../../../../assets/icons/notes/write-letter.svg");
const ADD_EVENT_SVG: &str = include_str!("../../../../assets/icons/notes/add-event.svg");
const EDIT_EVENT_SVG: &str = include_str!("../../../../assets/icons/notes/edit-event.svg");
const BACK_ARROW_SVG: &str =
    include_str!("../../../../assets/icons/notes/arrow-left-from-dotted-line.svg");
