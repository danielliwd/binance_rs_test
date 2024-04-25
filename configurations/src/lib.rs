#[allow(unused_imports)]
use log::{info, error, debug, warn, trace};
use structopt::StructOpt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::error::Error;
use std::fmt;

pub fn overwrite_x<T>(left: &mut T, right: T){
    *left=right;
}

pub fn parse()-> (Config, Opt){
    let opt = Opt::from_args();
    let conf = Config::load_yaml_with_opt_override(&opt).unwrap();
    (conf, opt)
}

#[derive(Debug)]
pub struct ConfigErr{
    err: String,
}
impl ConfigErr{
    pub fn new(err: &str) -> Box<Self> {
        Box::new(ConfigErr{err: err.to_string()})
    }
}
impl fmt::Display for ConfigErr{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(write!(f, "{}", self.err)?)
    }
}
impl Error for ConfigErr{}


#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config{
    version: usize,

    /// Whether to run this process in the background.
    pub daemon: bool,

    pub interval: u64,
    pub order_size_usd: u64,

    /// 最大下单次数
    pub max_order_count: u64,

    pub symbol: String,

    pub api_key: String,

    pub api_sec: String,
}

impl Default for Config{
    fn default() -> Self {
        Config{
            version: 0,
            daemon: false,
            interval: 1,
            order_size_usd: 10,
            max_order_count: 2,
            symbol: String::from(""),
            api_key: String::from(""),
            api_sec: String::from(""),
        }
    }
}


/// Call `Opt::from_args()` to build this object from the process's command line arguments.
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt{
    
    /// `-d` or `--daemon` can be used
    #[structopt(short, long)]
    pub daemon: bool,

    /// `-t` or `--test` can be used
    #[structopt(short, long)]
    pub test: bool,

    /// `-c` or `--conf` can be used
    #[structopt(short, long)]
    pub conf: Vec<String>,

}

impl Default for Opt {
    fn default() -> Self {
        Opt::from_args()
    }
}

fn merge_yaml(a: &mut serde_yaml::Value, b: serde_yaml::Value) {
    match (a, b) {
        (a @ &mut serde_yaml::Value::Mapping(_), serde_yaml::Value::Mapping(b)) => {
            let a = a.as_mapping_mut().unwrap();
            for (k, v) in b {
                if v.is_sequence() && a.contains_key(&k) && a[&k].is_sequence() { 
                    let mut _b = a.get(&k).unwrap().as_sequence().unwrap().to_owned();
                    _b.append(&mut v.as_sequence().unwrap().to_owned());
                    a[&k] = serde_yaml::Value::from(_b);
                    continue;
                }
                if !a.contains_key(&k) {a.insert(k.to_owned(), v.to_owned());}
                else { merge_yaml(&mut a[&k], v); }

            }
            
        }
        (a, b) => *a = b,
    }
}


impl Config{
    // Does not has to be async until we want runtime reload
    pub fn load_from_yaml<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<std::path::Path> + std::fmt::Display,
    {
        let conf_str = fs::read_to_string(&path)?;
        debug!("Conf file read from {path}");
        Self::from_yaml(&conf_str)
    }

    pub fn load_yaml_with_opt_override(opt: &Opt) -> Result<Self, Box<dyn Error>> {
        if opt.conf.len() == 0 {
            return Err(ConfigErr::new("No path specified"));
        }
        let mut target_yml: serde_yaml::Value = serde_yaml::from_str("---\nversion: 1")?;
        for ymlpath in &opt.conf {
            let conf_str = fs::read_to_string(&ymlpath)?;
            let val : serde_yaml::Value = serde_yaml::from_str(&conf_str)?;
            merge_yaml(&mut target_yml, val);
        }
        let mut conf: Self = serde_yaml::from_value(target_yml)?;

        if opt.daemon {
            conf.daemon = true;
        }
        Ok(conf)
    }

    pub fn new() -> Option<Self> {
        Self::from_yaml("---\nversion: 1").ok()
    }

    pub fn new_with_opt_override(opt: &Opt) -> Option<Self> {
        let conf = Self::new();
        match conf {
            Some(mut c) => {
                if opt.daemon {
                    c.daemon = true;
                }
                Some(c)
            }
            None => None,
        }
    }

    pub fn from_yaml(conf_str: &str) -> Result<Self, Box<dyn Error>> {
        trace!("Read conf file: {conf_str}");
        let conf: Self = serde_yaml::from_str(conf_str)?;
        trace!("Loaded conf: {conf:?}");
        conf.validate()
    }

    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }

    pub fn validate(self) -> Result<Self, Box<dyn Error>> {
        // TODO: do the validation
        Ok(self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn init_log() {
        let _ = env_logger::builder().is_test(true).try_init();
    }


    #[test]
    fn test_load_file() {
        init_log();
        let conf_str = r#"
---
version: 1
daemon: true
        "#
        .to_string();
        let conf = Config::from_yaml(&conf_str).unwrap();
        assert_eq!(true, conf.daemon);
        assert_eq!(1, conf.version);
    }
}
