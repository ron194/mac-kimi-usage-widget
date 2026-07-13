mod api;
mod config;
mod log;
mod prompt;
mod report;
mod usage;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");
const REFRESH_INTERVAL: Duration = Duration::from_secs(60);

fn load_icon() -> Icon {
    let image = image::load_from_memory(ICON_BYTES)
        .expect("embedded icon is valid PNG")
        .into_rgba8();
    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height).expect("icon rgba is valid")
}

struct MenuIds {
    refresh: tray_icon::menu::MenuId,
    settings: tray_icon::menu::MenuId,
    console: tray_icon::menu::MenuId,
    report: tray_icon::menu::MenuId,
    quit: tray_icon::menu::MenuId,
    menu: Menu,
}

fn add_local_usage_items(menu: &Menu, usage: &usage::AggregatedUsage) {
    let total = MenuItem::new(
        format!(
            "Total tokens: {}",
            usage::format_number(usage.total_tokens())
        ),
        false,
        None,
    );
    let output = MenuItem::new(
        format!("Output: {}", usage::format_number(usage.total_output)),
        false,
        None,
    );
    let input = MenuItem::new(
        format!(
            "Input (other): {}",
            usage::format_number(usage.total_input_other)
        ),
        false,
        None,
    );
    let cached = MenuItem::new(
        format!(
            "Cache read: {}",
            usage::format_number(usage.total_input_cache_read)
        ),
        false,
        None,
    );
    let today = MenuItem::new(
        format!("Today: {}", usage::format_number(usage.today_total)),
        false,
        None,
    );
    let week = MenuItem::new(
        format!(
            "Last 7 days: {}",
            usage::format_number(usage.last_7_days_total)
        ),
        false,
        None,
    );

    menu.append(&total).unwrap();
    menu.append(&output).unwrap();
    menu.append(&input).unwrap();
    menu.append(&cached).unwrap();
    menu.append(&PredefinedMenuItem::separator()).unwrap();
    menu.append(&today).unwrap();
    menu.append(&week).unwrap();
}

fn build_menu(
    api_quota: Option<&api::QuotaStats>,
    local_usage: &usage::AggregatedUsage,
    config: &config::Config,
    error: Option<&str>,
) -> MenuIds {
    let menu = Menu::new();

    let title = MenuItem::new("Kimi Code Usage", false, None);
    menu.append(&title).unwrap();
    menu.append(&PredefinedMenuItem::separator()).unwrap();

    if let Some(err) = error {
        let err_item = MenuItem::new(format!("Error: {err}"), false, None);
        menu.append(&err_item).unwrap();
        menu.append(&PredefinedMenuItem::separator()).unwrap();
    }

    if let Some(quota) = api_quota {
        let api_title = MenuItem::new("Console Quota (API)", false, None);
        let weekly = MenuItem::new(
            format!(
                "Weekly: {} / {} ({}%)",
                quota.weekly_used,
                quota.weekly_limit,
                quota.weekly_percentage()
            ),
            false,
            None,
        );
        let remaining = MenuItem::new(
            format!("Remaining: {}", quota.weekly_remaining),
            false,
            None,
        );
        let window = MenuItem::new(
            format!(
                "{}m window: {} / {}",
                quota.window_duration_minutes, quota.window_used, quota.window_limit
            ),
            false,
            None,
        );
        let reset = MenuItem::new(
            format!(
                "Reset: {}",
                quota.reset_time.as_deref().unwrap_or("unknown")
            ),
            false,
            None,
        );

        menu.append(&api_title).unwrap();
        menu.append(&weekly).unwrap();
        menu.append(&remaining).unwrap();
        menu.append(&window).unwrap();
        menu.append(&reset).unwrap();
        menu.append(&PredefinedMenuItem::separator()).unwrap();
    }

    let local_title = MenuItem::new("Local Token Usage", false, None);
    menu.append(&local_title).unwrap();
    add_local_usage_items(&menu, local_usage);

    let budget_item = MenuItem::new(
        format!(
            "Daily budget: {} ({}% today)",
            usage::format_number(config.daily_budget),
            config.percentage(local_usage.today_total)
        ),
        false,
        None,
    );
    menu.append(&PredefinedMenuItem::separator()).unwrap();
    menu.append(&budget_item).unwrap();

    let refresh = MenuItem::new("Refresh", true, None);
    let settings = MenuItem::new("Set API Key...", true, None);
    let console = MenuItem::new("Open Console...", true, None);
    let report = MenuItem::new("Open Usage Report...", true, None);
    let quit = MenuItem::new("Quit", true, None);

    menu.append(&PredefinedMenuItem::separator()).unwrap();
    menu.append(&refresh).unwrap();
    menu.append(&settings).unwrap();
    menu.append(&console).unwrap();
    menu.append(&report).unwrap();
    menu.append(&quit).unwrap();

    MenuIds {
        refresh: refresh.id().clone(),
        settings: settings.id().clone(),
        console: console.id().clone(),
        report: report.id().clone(),
        quit: quit.id().clone(),
        menu,
    }
}

#[derive(Clone)]
struct UiState {
    api_quota: Option<api::QuotaStats>,
    local_usage: usage::AggregatedUsage,
    refresh_id: tray_icon::menu::MenuId,
    settings_id: tray_icon::menu::MenuId,
    console_id: tray_icon::menu::MenuId,
    report_id: tray_icon::menu::MenuId,
    quit_id: tray_icon::menu::MenuId,
}

fn title_for_state(state: &UiState, config: &config::Config) -> String {
    if let Some(quota) = &state.api_quota {
        format!("Kimi {}%", quota.weekly_percentage())
    } else {
        format!("Kimi {}%", config.percentage(state.local_usage.today_total))
    }
}

fn update_ui(tray_icon: &TrayIcon, base_dir: &std::path::Path, config: &config::Config) -> UiState {
    let local_usage = usage::load_usage(base_dir);

    let (api_quota, error) = if let Some(key) = &config.api_key {
        match api::fetch_quota(key) {
            Ok(quota) => (Some(quota), None),
            Err(e) => (None, Some(e.to_string())),
        }
    } else {
        (
            None,
            Some(
                "No API key configured. Use 'Set API Key...' or set KIMI_CODE_API_KEY.".to_string(),
            ),
        )
    };

    let ids = build_menu(api_quota.as_ref(), &local_usage, config, error.as_deref());

    let state = UiState {
        api_quota,
        local_usage,
        refresh_id: ids.refresh,
        settings_id: ids.settings,
        console_id: ids.console,
        report_id: ids.report,
        quit_id: ids.quit,
    };

    tray_icon.set_menu(Some(Box::new(ids.menu)));
    tray_icon.set_title(Some(title_for_state(&state, config)));

    state
}

fn handle_set_api_key(config: &Rc<RefCell<config::Config>>) -> Option<String> {
    match prompt::prompt_api_key() {
        prompt::PromptResult::Value(key) => {
            let mut cfg = config.borrow_mut();
            match cfg.set_api_key(key) {
                Ok(()) => None,
                Err(e) => Some(format!("Failed to save API key: {e}")),
            }
        }
        prompt::PromptResult::Cancelled => None,
        prompt::PromptResult::Error(e) => Some(e),
    }
}

fn open_kimi_console() -> Result<(), Box<dyn std::error::Error>> {
    const URL: &str = "https://www.kimi.com/code/console";
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(URL).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(URL).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", URL])
            .spawn()?;
    }
    Ok(())
}

fn main() {
    config::ensure_default_config();
    let config = Rc::new(RefCell::new(config::Config::load()));

    let event_loop = EventLoopBuilder::new().build();

    let icon = load_icon();
    let base_dir = usage::kimi_code_dir().expect("could not determine home directory");

    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("Kimi Code Usage")
        .with_icon(icon)
        .build()
        .expect("failed to create tray icon");

    let state = RefCell::new(update_ui(&tray_icon, &base_dir, &config.borrow()));
    let tray_icon = Rc::new(tray_icon);
    let base_dir = Rc::new(base_dir);
    let mut next_refresh = Instant::now() + REFRESH_INTERVAL;

    let menu_channel = MenuEvent::receiver();

    event_loop.run(move |event, _event_loop, control_flow| {
        *control_flow = ControlFlow::WaitUntil(next_refresh);

        if let Event::NewEvents(StartCause::ResumeTimeReached { .. }) = event {
            let new_state = update_ui(&tray_icon, &base_dir, &config.borrow());
            *state.borrow_mut() = new_state;
            next_refresh = Instant::now() + REFRESH_INTERVAL;
        }

        if let Ok(event) = menu_channel.try_recv() {
            let id = event.id;
            let st = state.borrow();
            let is_refresh = id == st.refresh_id;
            let is_settings = id == st.settings_id;
            let is_console = id == st.console_id;
            let is_report = id == st.report_id;
            let is_quit = id == st.quit_id;
            drop(st);

            if is_refresh {
                let new_state = update_ui(&tray_icon, &base_dir, &config.borrow());
                *state.borrow_mut() = new_state;
                next_refresh = Instant::now() + REFRESH_INTERVAL;
            } else if is_settings {
                if let Some(err) = handle_set_api_key(&config) {
                    // Show error briefly by updating menu title.
                    tray_icon.set_title(Some(format!("Kimi: {err}")));
                } else {
                    // Reload config from disk to pick up the saved key and refresh UI.
                    if let Ok(path) = config_path() {
                        let reloaded = std::fs::read_to_string(&path)
                            .ok()
                            .and_then(|c| toml::from_str::<config::Config>(&c).ok())
                            .unwrap_or_else(config::Config::default);
                        *config.borrow_mut() = reloaded;
                    }
                    let new_state = update_ui(&tray_icon, &base_dir, &config.borrow());
                    *state.borrow_mut() = new_state;
                    next_refresh = Instant::now() + REFRESH_INTERVAL;
                }
            } else if is_console {
                if let Err(e) = open_kimi_console() {
                    tray_icon.set_title(Some(format!("Kimi: {e}")));
                }
            } else if is_report {
                if let Err(e) = report::open_usage_report(&base_dir) {
                    tray_icon.set_title(Some(format!("Kimi: {e}")));
                }
            } else if is_quit {
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}

fn config_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    config::config_path().ok_or_else(|| "could not determine config path".into())
}
