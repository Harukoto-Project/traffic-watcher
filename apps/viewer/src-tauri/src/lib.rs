mod database;
mod error;
mod events;
mod hourly;
mod overview;
mod range;
mod series;
mod snapshots;
mod state;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            database::commands::open_database,
            database::commands::close_database,
            database::commands::get_database_info,
            database::commands::find_default_database,
            overview::commands::load_overview,
            series::commands::load_series,
            snapshots::commands::list_snapshots,
            snapshots::commands::load_snapshot,
            events::commands::list_events,
            hourly::commands::list_hourly,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run the viewer application");
}
