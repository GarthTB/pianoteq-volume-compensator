use hound::WavReader;
use std::env::current_exe;
use std::fs::{File, read_to_string};
use std::io::BufReader;

#[derive(serde::Deserialize)]
pub(crate) struct Config {
    /// input file name (must be in the same directory as the executable)
    input_file: String,
    /// output binary file name (in the same directory as the executable, overwrite if exists)
    output_bin: String,
    /// output text file name (in the same directory as the executable, overwrite if exists)
    output_txt: String,
    /// start time of the targeted section in seconds
    pub(crate) start_time: f32,
    /// end time of the targeted section in seconds
    pub(crate) end_time: f32,
    /// number of adjacent notes considered on each side for averaging
    pub(crate) smooth_span: u8,
}

impl Config {
    pub(crate) fn new() -> Result<Self, &'static str> {
        let exe_path = current_exe().map_err(|_| "Failed to get executable path")?;
        let config_path = exe_path.with_file_name("config.toml");
        let config_str = read_to_string(config_path).map_err(|_| "Failed to read config file")?;
        toml::from_str(&config_str).map_err(|_| "Failed to parse config file")
    }

    pub(crate) fn get_input_reader(&self) -> Result<WavReader<BufReader<File>>, &'static str> {
        let exe_path = current_exe().map_err(|_| "Failed to get executable path")?;
        let input_file = exe_path.with_file_name(&self.input_file);
        WavReader::open(input_file).map_err(|_| "Failed to open input file")
    }

    pub(crate) fn get_output_bin_file(&self) -> Result<File, &'static str> {
        let exe_path = current_exe().map_err(|_| "Failed to get executable path")?;
        let output_bin = exe_path.with_file_name(&self.output_bin);
        if output_bin.exists() {
            std::fs::remove_file(&output_bin)
                .map_err(|_| "Failed to remove existing output binary file")?;
        }
        File::create(output_bin).map_err(|_| "Failed to create output binary file")
    }

    pub(crate) fn get_output_txt_file(&self) -> Result<File, &'static str> {
        let exe_path = current_exe().map_err(|_| "Failed to get executable path")?;
        let output_txt = exe_path.with_file_name(&self.output_txt);
        if output_txt.exists() {
            std::fs::remove_file(&output_txt)
                .map_err(|_| "Failed to remove existing output text file")?;
        }
        File::create(output_txt).map_err(|_| "Failed to create output binary file")
    }
}
