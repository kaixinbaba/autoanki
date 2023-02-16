#[allow(dead_code)]


use std::fs::File;
use std::path::Path;


const DEFAULT_CFG_NAME: &str = ".autoanki";


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
}

impl From<crate::cli::Args> for Config {
    fn from(args: crate::cli::Args) -> Self {

        let config_path = args.path.unwrap_or({
            #[cfg(not(target_os = "windows"))]
                let home = env!("HOME").to_string();
            #[cfg(target_os = "windows")]
                let home = env!("HOMEPATH").to_string();
            home
        });

        let file = File::open(Path::new(&config_path).join(DEFAULT_CFG_NAME)).unwrap();

        let config: Config = serde_json::from_reader(file).unwrap();

        config
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        let path = Path::new("123123");
        let path = path.join("aldfjaldfj");

        println!("{:?}", path);

    }

}