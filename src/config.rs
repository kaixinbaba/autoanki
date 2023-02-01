#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct Config {
    pub config_path: String,
}

impl From<crate::cli::Args> for Config {
    fn from(args: crate::cli::Args) -> Self {
        #[cfg(not(target_os = "windows"))]
        let home = env!("HOME").to_string();
        #[cfg(target_os = "windows")]
        let home = env!("HOMEPATH").to_string();

        let config_path = args.path.unwrap_or(home);
        Config {
            config_path,
        }
    }
}
