use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rlua::Lua;

pub const INTERCEPTOR_STR: &'static str = include_str!("interceptor.lua");

#[derive(Debug, Clone)]
pub struct FxManifest {
    pub ignored_paths: Vec<String>,
    pub ui_page: Option<String>,
}

impl FxManifest {
    fn new() -> Self {
        Self {
            ignored_paths: vec![
                ".git/**".into(),
                ".vscode/**".into(),
                ".gitattributes".into(),
            ],
            ui_page: None,
        }
    }
}

pub fn read_fxmanifest_file(path: &mut PathBuf) -> anyhow::Result<FxManifest> {
    let final_path = path.join("fxmanifest.lua");
    log::info!("Reading manifest from {}", final_path.display());

    let content = fs::read_to_string(final_path)?;

    let lua = Lua::new();
    let manifest = Arc::new(Mutex::new(FxManifest::new()));
    let manifest_clone = manifest.clone();

    let interceptor_function = lua.create_function(move |_, args: (String, String)| {
        let key = args.0;
        let value = args.1;
        log::debug!("Intercepted: {} -> {}", key, value);

        let mut manifest = manifest_clone.lock().unwrap();
        match key.as_str() {
            "ui_page" => {
                manifest.ui_page = Some(value);
            }
            "vx_ignore" => {
                manifest.ignored_paths.push(value);
            }
            _ => {}
        }

        Ok(())
    })?;

    lua.globals().set("_INTERCEPTOR", interceptor_function)?;
    lua.load(format!("{INTERCEPTOR_STR}\n{content}")).exec()?;

    let result = manifest.lock().unwrap();
    Ok(result.clone())
}
