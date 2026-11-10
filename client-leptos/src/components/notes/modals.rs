use super::utils::{CurrentIdn, ReadNotesMap, position_max, row_by_idn};
use crate::constants::{
    CLASS_CONFIRM, CLASS_ML_ACTIONS, CLASS_ML_BODY, CLASS_ML_BODY_FORM, CLASS_ML_CONTAINER,
    CLASS_ML_HEADER, CLASS_ML_WINDOW,
};
use crate::transport::send_data_conn;
use crate::utils::app_modal_error;
use leptos::{ev, html::*, prelude::*, reactive::signal::ArcRwSignal};
use shared::{DataConnParams, DataNoteAction, DataNoteEvent, DataNoteParams, DataNoteResults};
use std::sync::LazyLock;
use web_sys::js_sys;

type NotesModal = ArcRwSignal<Option<NotesModals>>;
pub static NOTES_MODAL: LazyLock<NotesModal> = LazyLock::new(|| ArcRwSignal::new(None));

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotesModals {
    EditItem,
    AddItem,
    RemoveItem,
    // --
    EditEvent,
    RemoveEvent,
    AddEvent,
    // --
    EditGroup,
    AddGroup,
    RemoveGroup,
}

pub fn notes_modal_view() -> impl IntoView {
    || NOTES_MODAL.get().map(modal_view)
}

fn modal_view(modal_type: NotesModals) -> impl IntoView {
    match modal_type {
        NotesModals::AddGroup => modal("Добавить группу", modal_group, modal_group_actions),
        NotesModals::EditGroup => modal("Свойства группы", modal_group, modal_group_actions),
        NotesModals::RemoveGroup => modal_remove("Удалить группу?"),
        // --
        NotesModals::AddItem => modal("Добавить запись", modal_item, modal_item_actions),
        NotesModals::EditItem => modal("Свойства записи", modal_item, modal_item_actions),
        NotesModals::RemoveItem => modal_remove("Удалить запись?"),
        // --
        NotesModals::AddEvent => modal_event("Добавить событие"),
        NotesModals::EditEvent => modal_event("Настройка события"),
        NotesModals::RemoveEvent => modal_event_remove("Удалить событие?"),
    }
}

pub fn get_current_idn() -> i32 {
    use_context::<CurrentIdn>()
        .expect("context::current_idn")
        .get_untracked()
}

pub fn get_current_row() -> DataNoteResults {
    row_by_idn(get_current_idn()).unwrap_or_default()
}

fn get_position_max(idn: i32) -> i32 {
    let notes_map = use_context::<ReadNotesMap>().expect("context::notes_map");
    position_max(&notes_map.get_untracked(), idn)
}

fn is_add_group() -> bool {
    NOTES_MODAL.get_untracked() == Some(NotesModals::AddGroup)
}

fn is_add_item() -> bool {
    NOTES_MODAL.get_untracked() == Some(NotesModals::AddItem)
}

fn input_string(data: RwSignal<String>) -> impl IntoView {
    input()
        .r#type("text")
        .value(data.get_untracked())
        .on(ev::change, move |e| {
            data.set(event_target_value(&e));
        })
}

fn input_position(position_max: i32) -> impl IntoView {
    let data = use_context::<ModalData>().expect("context::ModalData");
    input()
        .r#type("number")
        .prop("value", move || data.position.get())
        .min(1)
        .max(position_max)
        .on(ev::input, move |e| {
            let value = event_target_value(&e).parse::<i32>().unwrap_or_default();
            if value < 1 {
                data.position.set(1);
            } else if value > position_max {
                data.position.set(position_max);
            } else {
                data.position.set(value);
            }
        })
}

fn modal_group() -> AnyView {
    let data = use_context::<ModalData>().expect("context::ModalData");
    let mut position_max = get_position_max(0);
    if is_add_group() {
        data.label.set(String::new());
        position_max += 1;
        data.position.set(position_max);
    }
    div()
        .class(CLASS_ML_BODY_FORM)
        .child((input_string(data.label), input_position(position_max)))
        .into_any()
}

fn modal_group_actions() -> AnyView {
    let data = use_context::<ModalData>().expect("context::ModalData");
    (
        action_no(),
        button()
            .class(CLASS_CONFIRM)
            .child("да")
            .on(ev::click, move |_| {
                let label = data.label.get_untracked().trim().to_string();
                let position = data.position.get_untracked();
                if !(label.is_empty()) {
                    let prev = get_current_row();
                    let is_add = is_add_group();
                    send_data_conn(DataConnParams::Notes(DataNoteParams {
                        idn: if is_add { 0 } else { get_current_idn() },
                        label: (Some(label.clone()) != prev.label).then_some(label),
                        position: (Some(position) != prev.position).then_some(position),
                        action: if is_add {
                            DataNoteAction::Insert
                        } else {
                            DataNoteAction::Update
                        },
                        ..DataNoteParams::default()
                    }));
                }
                NOTES_MODAL.set(None);
            }),
    )
        .into_any()
}

fn modal_item() -> AnyView {
    let data = use_context::<ModalData>().expect("context::ModalData");
    let notes_map = use_context::<ReadNotesMap>().expect("context::notes_map");
    let current_idp = data.idp.get_untracked();
    let mut position_max = get_position_max(current_idp);
    if is_add_item() {
        data.label.set(String::new());
        position_max += 1;
        data.position.set(position_max);
    }
    let rows = notes_map.get().get(&0).cloned().unwrap_or_default();
    div()
        .class(CLASS_ML_BODY_FORM)
        .child((
            input_string(data.label),
            input_string(data.email),
            input_position(position_max),
            select()
                .on(ev::input, move |e| {
                    data.idp
                        .set(event_target_value(&e).parse::<i32>().unwrap_or_default());
                })
                .child(
                    rows.iter()
                        .map(|idn| {
                            let row = row_by_idn(*idn).unwrap_or_default();
                            option()
                                .selected(row.idn == current_idp)
                                .value(row.idn)
                                .child(row.label.unwrap_or_default())
                        })
                        .collect_view(),
                ),
        ))
        .into_any()
}

fn modal_item_actions() -> AnyView {
    let data = use_context::<ModalData>().expect("context::ModalData");
    (
        action_no(),
        button()
            .class(CLASS_CONFIRM)
            .child("да")
            .on(ev::click, move |_| {
                let label = data.label.get_untracked().trim().to_string();
                if !(label.is_empty()) {
                    let prev = get_current_row();
                    let is_add = is_add_item();
                    let email = data.email.get_untracked();
                    let idp = data.idp.get_untracked();
                    let position = if prev.idp == Some(idp) {
                        data.position.get_untracked()
                    } else {
                        get_position_max(idp) + 1
                    };
                    send_data_conn(DataConnParams::Notes(DataNoteParams {
                        idn: if is_add { 0 } else { get_current_idn() },
                        label: (Some(label.clone()) != prev.label).then_some(label),
                        email: (Some(email.clone()) != prev.email).then_some(email),
                        idp: (is_add || prev.idp != Some(idp)).then_some(idp),
                        position: (is_add || Some(position) != prev.position).then_some(position),
                        action: if is_add {
                            DataNoteAction::Insert
                        } else {
                            DataNoteAction::Update
                        },
                        ..DataNoteParams::default()
                    }));
                }
                NOTES_MODAL.set(None);
            }),
    )
        .into_any()
}

fn action_no() -> impl IntoView {
    button().child("нет").on(ev::click, |_| {
        NOTES_MODAL.set(None);
    })
}

#[derive(Clone)]
struct ModalData {
    pub label: RwSignal<String>,
    pub position: RwSignal<i32>,
    // --
    pub email: RwSignal<String>,
    pub idp: RwSignal<i32>,
}

fn modal_remove(title: &str) -> AnyView {
    div()
        .class(CLASS_ML_CONTAINER)
        .child(
            div().class(CLASS_ML_WINDOW).child((
                div().class(CLASS_ML_HEADER).child(title),
                div().class(CLASS_ML_ACTIONS).child((
                    action_no(),
                    button()
                        .class(CLASS_CONFIRM)
                        .child("да")
                        .on(ev::click, move |_| {
                            send_data_conn(DataConnParams::Notes(DataNoteParams {
                                idn: get_current_idn(),
                                action: DataNoteAction::Remove,
                                ..DataNoteParams::default()
                            }));
                            NOTES_MODAL.set(None);
                        }),
                )),
            )),
        )
        .into_any()
}

fn modal(
    title: &str,
    content: impl Fn() -> AnyView + 'static,
    actions: impl Fn() -> AnyView + 'static,
) -> AnyView {
    let row = get_current_row();

    provide_context::<ModalData>(ModalData {
        label: RwSignal::new(row.label.unwrap_or_default()),
        position: RwSignal::new(row.position.unwrap_or_default()),
        email: RwSignal::new(row.email.unwrap_or_default()),
        idp: RwSignal::new(if is_add_item() {
            row.idn
        } else {
            row.idp.unwrap_or_default()
        }),
    });

    div()
        .class(CLASS_ML_CONTAINER)
        .child(div().class(CLASS_ML_WINDOW).child((
            div().class(CLASS_ML_HEADER).child(title),
            div().class(CLASS_ML_BODY).child(content()),
            div().class(CLASS_ML_ACTIONS).child(actions()),
        )))
        .into_any()
}

const PERIOD_DAY: i32 = 1;
const PERIOD_MONTH: i32 = 2;
const PERIOD_YEAR: i32 = 3;

fn period_label(v: i32) -> String {
    match v {
        PERIOD_DAY => "день".to_string(),
        PERIOD_MONTH => "месяц".to_string(),
        PERIOD_YEAR => "год".to_string(),
        _ => "день".to_string(),
    }
}

fn modal_event(title: &str) -> AnyView {
    let row = get_current_row();
    let prev = row.event.unwrap_or_default();
    let is_edit = prev.is_some();
    let prev = if let Some(e) = prev {
        e
    } else {
        DataNoteEvent {
            date: js_sys::Date::new_0()
                .to_json()
                .as_string()
                .unwrap_or_default()[..10]
                .to_string(),
            delta: 1,
            period: 1,
        }
    };

    let date_signal = RwSignal::new(prev.date.clone());
    let delta_signal = RwSignal::new(prev.delta);
    let period_signal = RwSignal::new(prev.period);

    let periods = [PERIOD_DAY, PERIOD_MONTH, PERIOD_YEAR];

    div()
        .class(CLASS_ML_CONTAINER)
        .child(
            div().class(CLASS_ML_WINDOW).child((
                div().class(CLASS_ML_HEADER).child(title),
                div().class(CLASS_ML_BODY).child(
                    div().class(CLASS_ML_BODY_FORM).child((
                        input()
                            .r#type("date")
                            .value(date_signal.get_untracked())
                            .on(ev::change, move |e| {
                                date_signal.set(event_target_value(&e));
                            }),
                        input()
                            .r#type("number")
                            .prop("value", move || delta_signal.get())
                            .min(1)
                            .on(ev::input, move |e| {
                                let value =
                                    event_target_value(&e).parse::<i32>().unwrap_or_default();
                                if value < 1 {
                                    delta_signal.set(1);
                                } else {
                                    delta_signal.set(value);
                                }
                            }),
                        select()
                            .on(ev::input, move |e| {
                                period_signal
                                    .set(event_target_value(&e).parse::<i32>().unwrap_or_default());
                            })
                            .child(
                                periods
                                    .map(|val| {
                                        option()
                                            .selected(prev.period == val)
                                            .value(val)
                                            .child(period_label(val))
                                    })
                                    .collect_view(),
                            ),
                    )),
                ),
                div().class(CLASS_ML_ACTIONS).child((
                    is_edit.then_some(
                        button()
                            .child(move || {
                                format!(
                                    "+ {} {}",
                                    delta_signal.get(),
                                    period_label(period_signal.get())
                                )
                            })
                            .on(ev::click, move |_| {
                                if let Some(date) = to_date(date_signal.get_untracked()) {
                                    let event = DataNoteEvent {
                                        date,
                                        delta: delta_signal.get_untracked(),
                                        period: period_signal.get_untracked(),
                                    };
                                    send_data_conn(DataConnParams::Notes(DataNoteParams {
                                        idn: get_current_idn(),
                                        event: Some(DataNoteEvent {
                                            date: next_date(event.clone()),
                                            delta: event.delta,
                                            period: event.period,
                                        }),
                                        action: DataNoteAction::Update,
                                        ..DataNoteParams::default()
                                    }));
                                } else {
                                    app_modal_error("Некорректная дата");
                                }
                                NOTES_MODAL.set(None);
                            }),
                    ),
                    is_edit.then_some(button().child("удалить").on(ev::click, move |_| {
                        NOTES_MODAL.set(Some(NotesModals::RemoveEvent));
                    })),
                    action_no(),
                    button()
                        .class(CLASS_CONFIRM)
                        .child("да")
                        .on(ev::click, move |_| {
                            if let Some(date) = to_date(date_signal.get_untracked()) {
                                send_data_conn(DataConnParams::Notes(DataNoteParams {
                                    idn: get_current_idn(),
                                    event: Some(DataNoteEvent {
                                        date,
                                        delta: delta_signal.get_untracked(),
                                        period: period_signal.get_untracked(),
                                    }),
                                    action: DataNoteAction::Update,
                                    ..DataNoteParams::default()
                                }));
                            } else {
                                app_modal_error("Некорректная дата");
                            }
                            NOTES_MODAL.set(None);
                        }),
                )),
            )),
        )
        .into_any()
}

fn modal_event_remove(title: &str) -> AnyView {
    div()
        .class(CLASS_ML_CONTAINER)
        .child(
            div().class(CLASS_ML_WINDOW).child((
                div().class(CLASS_ML_HEADER).child(title),
                div().class(CLASS_ML_ACTIONS).child((
                    action_no(),
                    button()
                        .class(CLASS_CONFIRM)
                        .child("да")
                        .on(ev::click, move |_| {
                            send_data_conn(DataConnParams::Notes(DataNoteParams {
                                idn: get_current_idn(),
                                event: Some(DataNoteEvent::default()),
                                action: DataNoteAction::Update,
                                ..DataNoteParams::default()
                            }));
                            NOTES_MODAL.set(None);
                        }),
                )),
            )),
        )
        .into_any()
}

fn to_date(date: String) -> Option<String> {
    let js_date: js_sys::Date = js_sys::Date::new(&date.clone().into());
    if js_date.to_string() != *"Invalid Date"
        && js_date.to_json().to_string().substring(0, 10) == date
    {
        Some(date)
    } else {
        None
    }
}

fn next_date(event: DataNoteEvent) -> String {
    let js_date: js_sys::Date = js_sys::Date::new(&event.date.clone().into());
    let period = event.period;
    let delta = event.delta as u32;
    match period {
        PERIOD_DAY => {
            js_date.set_date(js_date.get_date() + delta);
        }
        PERIOD_MONTH => {
            js_date.set_month(js_date.get_month() + delta);
        }
        PERIOD_YEAR => {
            js_date.set_full_year(js_date.get_full_year() + delta);
        }
        _ => {}
    }
    js_date.to_json().substring(0, 10).into()
}
