use std::{
    io::Write,
    sync::atomic::{AtomicBool, Ordering},
};

use rquickjs::{CatchResultExt, Context, Function, Object, Runtime, Value};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn print(s: String) {
    println!("{s}");
}

#[tauri::command]
fn run_js() -> Result<(), String> {
    let rt = Runtime::new().map_err(|e| e.to_string())?;
    let ctx = Context::full(&rt).map_err(|e| e.to_string())?;
    let should_exit = std::sync::Arc::new(AtomicBool::new(false));

    ctx.with(|ctx| -> Result<(), String> {
        let global = ctx.globals();
        let should_exit_clone = should_exit.clone();

        global
            .set(
                "__print",
                Function::new(ctx.clone(), print)
                    .map_err(|e| e.to_string())?
                    .with_name("__print")
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        global
            .set(
                "exit",
                Function::new(ctx.clone(), move || {
                    should_exit_clone.store(true, Ordering::Relaxed);
                })
                .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        ctx.eval::<(), _>(
            r#"
globalThis.console = {
  log(...v) {
    globalThis.__print(`${v.join(" ")}`)
  }
}
"#,
        )
        .map_err(|e| e.to_string())?;

        let console: Object = global.get("console").map_err(|e| e.to_string())?;
        let js_log: Function = console.get("log").map_err(|e| e.to_string())?;
        loop {
            if should_exit.load(Ordering::Relaxed) {
                break;
            }
            let mut input = String::new();
            print!("> ");
            std::io::stdout().flush().map_err(|e| e.to_string())?;
            std::io::stdin()
                .read_line(&mut input)
                .map_err(|e| e.to_string())?;
            ctx.eval::<Value, _>(input.as_bytes())
                .and_then(|ret| js_log.call::<(Value<'_>,), ()>((ret,)))
                .catch(&ctx)
                .unwrap_or_else(|err| println!("{err}"));
        }
        Ok(())
    })?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, run_js])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
