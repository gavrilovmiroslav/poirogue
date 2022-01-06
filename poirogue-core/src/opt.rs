use structopt::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "Poirogue")]
pub struct Opt {
    #[structopt(short = "seed", long, default_value = "0")]
    pub random_seed: u64,
    #[structopt(short = "r", long)]
    pub release_mode: bool,
    #[structopt(short = "b", long)]
    pub skip_binarize_on_boot: bool,
    #[structopt(short = "d", long, default_value = "resources/data")]
    pub data_directory: String,
    #[structopt(short = "l", long, default_value = "3")]
    pub log_height: usize,
    #[structopt(short = "t", long, default_value = "360")]
    pub log_expiry: u32,
}