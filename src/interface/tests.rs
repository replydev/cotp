use super::{
    app::App,
    enums::{Focus, Page},
    handlers::{handle_key_events, handle_mouse_events, handle_paste},
    row::{code_for, remaining},
};
use crate::otp::{
    otp_algorithm::OTPAlgorithm,
    otp_element::{OTPDatabase, OTPElement},
    otp_type::OTPType,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

fn account(issuer: &str, label: &str) -> OTPElement {
    OTPElement {
        id: String::new(),
        alias: None,
        tags: Vec::new(),
        favorite: false,
        order: None,
        secret: "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".into(),
        issuer: issuer.into(),
        label: label.into(),
        digits: 6,
        type_: OTPType::Totp,
        algorithm: OTPAlgorithm::Sha1,
        period: 30,
        counter: None,
        pin: None,
    }
}

fn database() -> OTPDatabase {
    OTPDatabase {
        elements: vec![
            account("GitHub", "alex@studio.dev"),
            account("Google", "alex@example.com"),
            account("Linear", "Design team"),
        ],
        ..Default::default()
    }
}

fn key(app: &mut App, code: KeyCode) {
    handle_key_events(KeyEvent::new(code, KeyModifiers::NONE), app);
}

fn render(app: &mut App, width: u16, height: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    terminal.backend().buffer().clone()
}

fn text(buffer: &Buffer) -> String {
    buffer.content.iter().map(|cell| cell.symbol()).collect()
}

#[test]
fn filtering_tracks_database_identity_and_handles_no_results() {
    let mut db = database();
    let mut app = App::new(&mut db);
    key(&mut app, KeyCode::Char('/'));
    handle_paste("GOOGLE", &mut app);
    assert_eq!(app.table.items.len(), 1);
    assert_eq!(app.table.selected_index(), Some(1));
    app.tick(true);
    assert_eq!(app.table.selected_index(), Some(1));
    key(&mut app, KeyCode::Char('x'));
    assert!(app.table.items.is_empty());
    assert_eq!(app.table.selected_index(), None);
    assert!(text(&render(&mut app, 100, 24)).contains("No matching accounts"));
    key(&mut app, KeyCode::Backspace);
    assert_eq!(app.table.selected_index(), Some(1));
    key(&mut app, KeyCode::Esc);
    key(&mut app, KeyCode::Esc);
    assert_eq!(app.table.items.len(), 3);
    assert_eq!(app.table.selected_index(), Some(1));
}

#[test]
fn deletion_and_counter_updates_target_the_filtered_account() {
    let mut db = database();
    db.elements[1].type_ = OTPType::Hotp;
    db.elements[1].counter = Some(4);
    let mut app = App::new(&mut db);
    app.search_query = "Google".into();
    app.filter();
    key(&mut app, KeyCode::Char('+'));
    assert_eq!(app.database.elements[1].counter, Some(5));
    assert_eq!(app.database.elements[0].counter, None);
    key(&mut app, KeyCode::Char('d'));
    assert_eq!(app.focus, Focus::Popup);
    key(&mut app, KeyCode::Char('y'));
    assert_eq!(app.database.elements.len(), 2);
    assert_eq!(app.database.elements[1].issuer, "Linear");
    assert!(app.table.items.is_empty());
    assert_eq!(app.table.state.selected(), None);
    assert!(app.database.is_modified());
}

#[test]
fn deleting_last_account_leaves_a_safe_empty_state() {
    let mut db = OTPDatabase {
        elements: vec![account("Example", "alice")],
        ..Default::default()
    };
    let mut app = App::new(&mut db);
    key(&mut app, KeyCode::Char('d'));
    key(&mut app, KeyCode::Char('y'));
    key(&mut app, KeyCode::Up);
    key(&mut app, KeyCode::Down);
    assert_eq!(app.table.selected_index(), None);
    assert!(text(&render(&mut app, 120, 30)).contains("Your vault is ready"));
}

#[test]
fn countdown_uses_the_account_period_and_steam_fixed_period() {
    let mut element = account("Example", "alice");
    element.period = 45;
    assert_eq!(remaining(&element, 44), Some((1, 45)));
    assert_eq!(remaining(&element, 45), Some((45, 45)));
    element.type_ = OTPType::Steam;
    assert_eq!(remaining(&element, 45), Some((15, 30)));
    element.type_ = OTPType::Hotp;
    assert_eq!(remaining(&element, 45), None);
    element.type_ = OTPType::Totp;
    element.period = 0;
    assert_eq!(remaining(&element, 45), None);
    assert!(code_for(&element).is_err());
}

#[test]
fn refresh_updates_codes_without_resetting_filter_or_selection() {
    let mut db = database();
    let mut app = App::new(&mut db);
    app.search_query = "alex".into();
    app.filter();
    app.table.next();
    app.table.items[1].code = "stale".into();
    app.now = 0;
    app.tick(false);
    assert_eq!(app.table.selected_index(), Some(1));
    assert_eq!(app.table.items.len(), 2);
    assert_ne!(app.table.items[1].code, "stale");
}

#[test]
fn qr_escape_returns_to_accounts_and_copy_errors_are_reported() {
    let mut db = database();
    let mut app = App::new(&mut db);
    key(&mut app, KeyCode::Char(' '));
    assert_eq!(app.current_page, Page::Qrcode);
    key(&mut app, KeyCode::Esc);
    assert!(app.running);
    assert_eq!(app.current_page, Page::Main);
    app.database.elements[0].secret = "INVALID!".into();
    key(&mut app, KeyCode::Enter);
    assert!(app.notification.as_ref().unwrap().1);
    assert!(
        app.notification
            .as_ref()
            .unwrap()
            .0
            .starts_with("Could not copy:")
    );
}

#[test]
fn plain_c_does_not_delete_and_save_confirmation_can_be_cancelled() {
    let mut db = database();
    let mut app = App::new(&mut db);
    key(&mut app, KeyCode::Char('c'));
    assert_eq!(app.focus, Focus::MainPage);
    app.database.mark_modified();
    key(&mut app, KeyCode::Char('q'));
    assert_eq!(app.focus, Focus::Popup);
    key(&mut app, KeyCode::Esc);
    assert!(app.running);
    key(&mut app, KeyCode::Char('q'));
    key(&mut app, KeyCode::Char('n'));
    assert!(!app.running);
    assert!(!app.database.is_modified());
}

#[test]
fn search_supports_unicode_paste_and_mouse_navigation() {
    let mut db = database();
    db.elements[0].label = "東京のアカウント".into();
    let mut app = App::new(&mut db);
    key(&mut app, KeyCode::Char('/'));
    handle_paste("東京\n", &mut app);
    assert_eq!(app.search_query, "東京");
    assert_eq!(app.table.items.len(), 1);
    handle_key_events(
        KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
        &mut app,
    );
    assert_eq!(app.table.items.len(), 3);
    handle_mouse_events(
        MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 10,
            row: 10,
            modifiers: KeyModifiers::NONE,
        },
        &mut app,
    );
    assert_eq!(app.table.selected_index(), Some(1));
    assert_eq!(app.focus, Focus::SearchBar);
}

#[test]
fn hidden_codes_and_pins_never_appear_on_dashboard() {
    let mut db = database();
    db.elements[0].pin = Some("sensitive-pin".into());
    let mut app = App::new(&mut db);
    app.table.items[0].code = "123456".into();
    key(&mut app, KeyCode::Char('h'));
    let output = text(&render(&mut app, 120, 30));
    assert!(!output.contains("123 456"));
    assert!(!output.contains("sensitive-pin"));
    assert!(output.contains("Codes hidden"));
}

#[test]
fn layouts_render_across_sizes_and_overlay_states() {
    let mut db = database();
    let mut app = App::new(&mut db);
    for (width, height) in [
        (1, 1),
        (20, 5),
        (36, 12),
        (60, 18),
        (80, 24),
        (120, 30),
        (180, 50),
    ] {
        render(&mut app, width, height);
        key(&mut app, KeyCode::Char('?'));
        render(&mut app, width, height);
        key(&mut app, KeyCode::Esc);
        key(&mut app, KeyCode::Char(' '));
        render(&mut app, width, height);
        key(&mut app, KeyCode::Esc);
        key(&mut app, KeyCode::Char('/'));
        app.search_query = "東京👩‍💻".repeat(100);
        render(&mut app, width, height);
        app.search_query.clear();
        key(&mut app, KeyCode::Esc);
    }
}

#[test]
fn account_scroll_keeps_last_selection_visible() {
    let mut db = database();
    for i in 0..40 {
        db.elements
            .push(account(&format!("Account {i:02}"), "alice"));
    }
    let mut app = App::new(&mut db);
    key(&mut app, KeyCode::End);
    assert!(text(&render(&mut app, 80, 24)).contains("Account 39"));
    key(&mut app, KeyCode::Home);
    assert!(text(&render(&mut app, 80, 24)).contains("GitHub"));
}

#[test]
fn dashboard_preview() {
    let mut db = database();
    db.elements.extend([
        account("AWS", "Production · engineering"),
        account("Figma", "alex@studio.dev"),
        account("Proton", "Personal"),
    ]);
    let mut app = App::new(&mut db);
    app.now = 12;
    for (i, item) in app.table.items.iter_mut().enumerate() {
        item.code = format!("{:06}", 751109 + i * 7319);
    }
    let buffer = render(&mut app, 120, 30);
    let output = text(&buffer);
    assert!(output.contains("751 109"));
    assert!(output.contains("Selected account"));
    assert!(output.contains("18s"));
    if let Ok(path) = std::env::var("COTP_UI_PREVIEW") {
        write_preview_svg(&buffer, &path);
    }
}

// Optional documentation preview generated from the actual rendered buffer and fake accounts.
fn write_preview_svg(buffer: &Buffer, path: &str) {
    use ratatui::style::Color;
    fn color(value: Color) -> String {
        match value {
            Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
            Color::Black => "#000000".into(),
            _ => "#e0e9f2".into(),
        }
    }
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\"><title>cotp dashboard with demonstration accounts</title><rect width=\"100%\" height=\"100%\" fill=\"#0d131d\"/>",
        buffer.area.width * 9,
        buffer.area.height * 18,
        buffer.area.width * 9,
        buffer.area.height * 18
    );
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = &buffer[(x, y)];
            let symbol = cell
                .symbol()
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            svg.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"9\" height=\"18\" fill=\"{}\"/>",
                x * 9,
                y * 18,
                color(cell.bg)
            ));
            if !symbol.trim().is_empty() {
                svg.push_str(&format!("<text x=\"{}\" y=\"{}\" fill=\"{}\" font-family=\"DejaVu Sans Mono,monospace\" font-size=\"14\">{symbol}</text>", x * 9, y * 18 + 14, color(cell.fg)));
            }
        }
    }
    svg.push_str("</svg>\n");
    std::fs::write(path, svg).unwrap();
}

fn control(app: &mut App, c: char) {
    handle_key_events(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL), app);
}

fn click(app: &mut App, action: super::enums::Action) {
    let area = app
        .hitboxes
        .iter()
        .find(|(_, candidate)| *candidate == action)
        .expect("action must be visible")
        .0;
    handle_mouse_events(
        MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: area.x,
            row: area.y,
            modifiers: KeyModifiers::NONE,
        },
        app,
    );
}

#[test]
fn add_account_entirely_with_keyboard_and_save_button() {
    let mut db = OTPDatabase::default();
    let mut app = App::new(&mut db);
    key(&mut app, KeyCode::Char('a'));
    assert_eq!(app.focus, Focus::Form);
    handle_paste("Example", &mut app);
    key(&mut app, KeyCode::Tab);
    handle_paste("alice@example.com", &mut app);
    key(&mut app, KeyCode::Tab);
    handle_paste("gezd gnbv gy3t qojq gezd gnbv gy3t qojq", &mut app);
    // Tab through type, algorithm, digits and period to the explicit Save button.
    for _ in 0..5 {
        key(&mut app, KeyCode::Tab);
    }
    assert_eq!(app.form.as_ref().unwrap().active, super::form::SAVE);
    key(&mut app, KeyCode::Enter);
    assert_eq!(app.focus, Focus::MainPage);
    assert!(app.form.is_none());
    assert_eq!(app.database.elements.len(), 1);
    assert_eq!(app.database.elements[0].issuer, "Example");
    assert_eq!(app.database.elements[0].label, "alice@example.com");
    assert_eq!(
        app.database.elements[0].secret,
        "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
    );
    assert!(app.database.is_modified());
}

#[test]
fn editing_filtered_account_keeps_secret_and_pin_and_supports_cursor_editing() {
    let mut db = database();
    db.elements[1].type_ = OTPType::Motp;
    db.elements[1].secret = "e3152afee62599c8".into();
    db.elements[1].pin = Some("1234".into());
    let mut app = App::new(&mut db);
    app.search_query = "Google".into();
    app.filter();
    key(&mut app, KeyCode::Char('e'));
    key(&mut app, KeyCode::Home);
    handle_paste("My ", &mut app);
    control(&mut app, 's');
    assert_eq!(app.database.elements[1].issuer, "My Google");
    assert_eq!(app.database.elements[0].issuer, "GitHub");
    assert_eq!(app.database.elements[1].secret, "e3152afee62599c8");
    assert_eq!(app.database.elements[1].pin.as_deref(), Some("1234"));
    assert_eq!(app.table.selected_index(), Some(1));
}

#[test]
fn invalid_form_focuses_problem_and_never_mutates_database() {
    let mut db = database();
    let mut app = App::new(&mut db);
    key(&mut app, KeyCode::Char('e'));
    app.form.as_mut().unwrap().inputs[6] = tui_input::Input::from("0");
    control(&mut app, 's');
    let form = app.form.as_ref().unwrap();
    assert_eq!(form.active, 6);
    assert!(form.error.as_ref().unwrap().contains("Period"));
    assert!(!app.database.is_modified());
    app.form.as_mut().unwrap().inputs[6] = tui_input::Input::from("45");
    app.form.as_mut().unwrap().inputs[5] = tui_input::Input::from("11");
    control(&mut app, 's');
    assert_eq!(app.form.as_ref().unwrap().active, 5);
    assert!(!app.database.is_modified());
    key(&mut app, KeyCode::Esc);
    assert_eq!(app.database.elements[0].period, 30);
    assert!(app.form.is_none());
}

#[test]
fn hotp_form_validates_counter_and_applies_it() {
    let mut db = database();
    let mut app = App::new(&mut db);
    app.open_form(true);
    let form = app.form.as_mut().unwrap();
    form.active = 3;
    form.cycle(false);
    assert!(form.fields().contains(&7));
    assert!(!form.fields().contains(&6));
    form.inputs[7] = tui_input::Input::from("-1");
    control(&mut app, 's');
    assert_eq!(app.form.as_ref().unwrap().active, 7);
    app.form.as_mut().unwrap().inputs[7] = tui_input::Input::from("12");
    control(&mut app, 's');
    assert_eq!(app.database.elements[0].type_, OTPType::Hotp);
    assert_eq!(app.database.elements[0].counter, Some(12));
}

#[test]
fn pasted_uri_populates_form_and_secret_remains_masked() {
    let mut db = OTPDatabase::default();
    let mut app = App::new(&mut db);
    app.open_form(false);
    app.form.as_mut().unwrap().active = 2;
    handle_paste(
        "otpauth://totp/Example:alice?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&period=45&digits=8",
        &mut app,
    );
    let output = text(&render(&mut app, 100, 30));
    assert!(!output.contains("GEZDGNBV"));
    assert!(output.contains("masked"));
    control(&mut app, 's');
    assert_eq!(app.database.elements[0].period, 45);
    assert_eq!(app.database.elements[0].digits, 8);
    assert_eq!(app.database.elements[0].issuer, "Example");
}

#[test]
fn form_rejects_invalid_secret_and_missing_pin_without_saving() {
    let mut db = database();
    let mut app = App::new(&mut db);
    app.open_form(true);
    app.form.as_mut().unwrap().active = 2;
    handle_paste("invalid!", &mut app);
    control(&mut app, 's');
    assert_eq!(app.form.as_ref().unwrap().active, 2);
    control(&mut app, 'u');
    app.form.as_mut().unwrap().inputs[3] = tui_input::Input::from("YANDEX");
    control(&mut app, 's');
    assert_eq!(app.form.as_ref().unwrap().active, 8);
    assert!(!app.database.is_modified());
    control(&mut app, 'c');
    assert!(app.running);
    assert!(app.form.is_none());
}

#[test]
fn mouse_actions_select_search_edit_and_cancel_without_clicking_through() {
    use super::enums::Action;
    let mut db = database();
    let mut app = App::new(&mut db);
    render(&mut app, 120, 30);
    click(&mut app, Action::Select(1));
    assert_eq!(app.table.selected_index(), Some(1));
    click(&mut app, Action::Search);
    handle_paste("Google", &mut app);
    render(&mut app, 120, 30);
    click(&mut app, Action::Command(KeyCode::Char('e')));
    assert_eq!(app.focus, Focus::Form);
    render(&mut app, 120, 30);
    assert!(
        !app.hitboxes
            .iter()
            .any(|(_, action)| matches!(action, Action::Select(_)))
    );
    click(&mut app, Action::Field(1));
    control(&mut app, 'u');
    handle_paste("modified", &mut app);
    click(&mut app, Action::Cancel);
    assert_eq!(app.database.elements[1].label, "alex@example.com");
    assert!(!app.database.is_modified());
}

#[test]
fn mouse_can_add_fill_and_save_an_account() {
    use super::enums::Action;
    let mut db = OTPDatabase::default();
    let mut app = App::new(&mut db);
    render(&mut app, 120, 34);
    click(&mut app, Action::Command(KeyCode::Char('a')));
    render(&mut app, 120, 34);
    click(&mut app, Action::Field(1));
    handle_paste("alice", &mut app);
    click(&mut app, Action::Field(2));
    handle_paste("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ", &mut app);
    click(&mut app, Action::Save);
    assert_eq!(app.database.elements.len(), 1);
    assert_eq!(app.database.elements[0].label, "alice");
}

#[test]
fn forms_keep_focused_fields_visible_at_every_supported_size() {
    let mut db = database();
    let mut app = App::new(&mut db);
    app.open_form(true);
    for (width, height) in [(36, 12), (60, 18), (80, 24), (120, 34)] {
        for index in app.form.as_ref().unwrap().fields() {
            app.form.as_mut().unwrap().active = index;
            let output = text(&render(&mut app, width, height));
            assert!(
                output.contains(super::form::LABELS[index]),
                "focused field {index} missing at {width}x{height}"
            );
        }
        let output = text(&render(&mut app, width, height));
        assert!(output.contains("Save account"));
        assert!(output.contains("Cancel"));
    }
}

#[test]
fn editor_preview() {
    let mut db = database();
    let mut app = App::new(&mut db);
    app.now = 12;
    for (i, item) in app.table.items.iter_mut().enumerate() {
        item.code = format!("{:06}", 751109 + i * 7319);
    }
    app.open_form(true);
    app.form.as_mut().unwrap().active = 1;
    let buffer = render(&mut app, 120, 34);
    if let Ok(path) = std::env::var("COTP_FORM_PREVIEW") {
        write_preview_svg(&buffer, &path);
    }
}

#[test]
fn clickable_help_exposes_actions_on_narrow_terminals() {
    use super::enums::Action;
    let mut db = database();
    let mut app = App::new(&mut db);
    key(&mut app, KeyCode::Char('?'));
    app.popup_scroll = 10;
    render(&mut app, 36, 18);
    click(&mut app, Action::Command(KeyCode::Char(' ')));
    assert_eq!(app.current_page, Page::Qrcode);
    assert_eq!(app.focus, Focus::MainPage);
    render(&mut app, 36, 18);
    click(&mut app, Action::Key(KeyCode::Esc));
    assert_eq!(app.current_page, Page::Main);
    assert!(app.running);
}

#[test]
fn minimum_terminal_size_still_displays_a_selected_code() {
    let mut db = database();
    let mut app = App::new(&mut db);
    app.table.items[0].code = "123456".into();
    let output = text(&render(&mut app, 36, 12));
    assert!(output.contains("GitHub"));
    assert!(output.contains("123 456"));
}

#[test]
fn keyboard_and_mouse_navigation_stop_at_both_boundaries() {
    let mut db = database();
    let mut app = App::new(&mut db);
    for backwards in [true, false] {
        key(
            &mut app,
            if backwards {
                KeyCode::Home
            } else {
                KeyCode::End
            },
        );
        let expected = if backwards {
            0
        } else {
            app.table.items.len() - 1
        };
        for _ in 0..3 {
            key(
                &mut app,
                if backwards {
                    KeyCode::Up
                } else {
                    KeyCode::Down
                },
            );
            handle_mouse_events(
                MouseEvent {
                    kind: if backwards {
                        MouseEventKind::ScrollUp
                    } else {
                        MouseEventKind::ScrollDown
                    },
                    column: 10,
                    row: 10,
                    modifiers: KeyModifiers::NONE,
                },
                &mut app,
            );
            assert_eq!(app.table.state.selected(), Some(expected));
        }
    }
    app.search_query = "Google".into();
    app.filter();
    app.table.next();
    app.table.previous();
    assert_eq!(app.table.selected_index(), Some(1));
}

#[test]
fn countdown_is_shown_once_and_header_does_not_clip_status() {
    let mut db = database();
    let mut app = App::new(&mut db);
    app.now = 12;
    for width in [36, 80, 120] {
        let output = text(&render(&mut app, width, 24));
        assert_eq!(output.matches("18s").count(), 1);
        assert!(!output.contains("VALID FOR"));
        assert!(!output.contains("vault open"));
        assert!(output.contains("3 accounts"));
    }
    app.database.mark_modified();
    let buffer = render(&mut app, 36, 24);
    let header: String = (0..36).map(|x| buffer[(x, 1)].symbol()).collect();
    assert!(header.contains("3 accounts  ● unsaved"));
}

/// Scripted recording of the real widgets and handlers with demonstration data only.
/// Run through ci/render_ui_demo.py; excluded from ordinary verification runs.
#[test]
#[ignore = "generates frames for the README animation"]
fn record_ui_demo() {
    let directory = std::env::var("COTP_UI_DEMO_DIR").expect("set the frame output directory");
    let mut db = database();
    db.elements.extend([
        account("AWS", "Production · engineering"),
        account("Figma", "alex@studio.dev"),
    ]);
    let mut app = App::new(&mut db);
    let mut frames = Vec::new();
    let mut elapsed = 0;
    let mut record = |app: &mut App, caption: &str, delay: u64| {
        app.now = 12 + elapsed / 100;
        for item in &mut app.table.items {
            item.code = format!(
                "{:06}",
                751109 + item.database_index * 7319 + (app.now / 30) as usize * 1023
            );
        }
        let filename = format!("frame-{:03}.svg", frames.len());
        let buffer = render(app, 120, 34);
        write_preview_svg(&buffer, &format!("{directory}/{filename}"));
        frames.push(serde_json::json!({"file": filename, "caption": caption, "delay": delay}));
        elapsed += delay;
    };

    record(&mut app, "Your accounts, at a glance", 200);
    key(&mut app, KeyCode::Down);
    record(
        &mut app,
        "Arrow keys or mouse wheel to select an account",
        100,
    );
    key(&mut app, KeyCode::Char('/'));
    record(&mut app, "Press / to find an account", 50);
    for c in "google".chars() {
        key(&mut app, KeyCode::Char(c));
        record(&mut app, "Filter accounts as you type", 12);
    }
    record(&mut app, "Filter accounts as you type", 100);
    key(&mut app, KeyCode::Esc);
    key(&mut app, KeyCode::Char('e'));
    record(&mut app, "Press e to edit the selected account", 100);
    key(&mut app, KeyCode::Tab);
    control(&mut app, 'u');
    handle_paste("work@example.com", &mut app);
    record(
        &mut app,
        "Tab between fields; existing secrets stay masked",
        150,
    );
    control(&mut app, 's');
    record(&mut app, "Ctrl+S applies your changes", 150);
    app.notification = None;

    // Use the same hitboxes as a real mouse click to open the Add form.
    render(&mut app, 120, 34);
    click(&mut app, super::enums::Action::Command(KeyCode::Char('a')));
    record(&mut app, "Click Add, or press a, to create an account", 80);
    handle_paste("Proton", &mut app);
    key(&mut app, KeyCode::Tab);
    record(&mut app, "Enter the service and account name", 60);
    handle_paste("personal@example.com", &mut app);
    key(&mut app, KeyCode::Tab);
    record(&mut app, "Enter the service and account name", 60);
    handle_paste("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ", &mut app);
    record(&mut app, "Paste a secret key or an otpauth:// URI", 100);
    control(&mut app, 's');
    record(&mut app, "Your new account is ready", 180);
    app.notification = None;
    key(&mut app, KeyCode::Char('h'));
    record(&mut app, "Press h to hide codes", 100);
    key(&mut app, KeyCode::Char('h'));
    record(&mut app, "Press h again to reveal them", 70);
    key(&mut app, KeyCode::Char('?'));
    record(&mut app, "Press ? for shortcuts and clickable actions", 140);
    key(&mut app, KeyCode::Esc);
    key(&mut app, KeyCode::Char('q'));
    record(
        &mut app,
        "Confirm on exit to save changes to the encrypted vault",
        200,
    );
    key(&mut app, KeyCode::Esc);
    key(&mut app, KeyCode::Home);
    record(&mut app, "cotp · built for your terminal", 180);
    std::fs::write(
        format!("{directory}/frames.json"),
        serde_json::to_string_pretty(&frames).unwrap(),
    )
    .unwrap();
}

#[test]
fn locking_wipes_plaintext_and_key_and_restores_pending_database_and_form() {
    use crate::{
        crypto::cryptography::{argon_derive_key, gen_salt},
        persistence,
    };
    use zeroize::Zeroizing;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault");
    std::fs::write(&path, b"unchanged disk").unwrap();
    persistence::initialize(&path).unwrap();
    let salt = gen_salt().unwrap();
    let key = Zeroizing::new(argon_derive_key(b"password", &salt).unwrap());
    persistence::unlock(&key, &salt).unwrap();
    let mut database = database();
    database.mark_modified();
    let mut app = App::new(&mut database);
    app.open_form(false);
    app.form.as_mut().unwrap().active = 2;
    handle_paste("MYUNFINISHEDSECRET", &mut app);
    handle_key_events(
        KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL),
        &mut app,
    );
    assert!(app.sealed.is_some());
    assert!(app.database.elements_ref().is_empty());
    assert!(!app.database.is_modified());
    assert!(app.table.items.is_empty());
    assert!(app.form.is_none());
    assert!(app.search_query.is_empty());
    assert!(persistence::seal("cannot encrypt without key").is_err());
    assert!(!app.sealed.as_ref().unwrap().contains("MYUNFINISHEDSECRET"));
    let locked = text(&render(&mut app, 100, 20));
    assert!(locked.contains("Locked"));
    assert!(!locked.contains("alex@"));
    app.unlock_password.push_str("wrong password");
    app.unlock_session();
    assert!(app.sealed.is_some());
    assert!(app.unlock_password.is_empty());
    app.unlock_password.push_str("password");
    app.unlock_session();
    assert!(app.sealed.is_none());
    assert_eq!(app.database.elements_ref().len(), 3);
    assert!(app.database.is_modified());
    assert_eq!(app.focus, Focus::Form);
    let form_state = serde_json::to_string(&app.form.as_ref().unwrap().snapshot()).unwrap();
    assert!(form_state.contains("MYUNFINISHEDSECRET"));
    assert_eq!(std::fs::read(&path).unwrap(), b"unchanged disk");
    app.cancel_form();
    app.last_activity = std::time::Instant::now() - std::time::Duration::from_secs(301);
    app.tick(false);
    assert!(app.sealed.is_some());
    assert!(persistence::seal("locked again").is_err());
}
