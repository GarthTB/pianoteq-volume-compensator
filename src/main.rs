mod config;
mod utils;

fn main() {
    println!("Pianoteq Volume Compensator");
    println!("Version: 0.1.0");
    println!("Author: Garth TB <g-art-h@outlook.com>");
    println!("Repo: https://github.com/garthtb/pianoteq-volume-compensator");

    println!("Loading configuration...");
    let config = config::Config::new().unwrap_or_else(|msg| abort(&msg));
    let input_reader = config.get_input_reader().unwrap_or_else(|msg| abort(&msg));
    let sample_rate = input_reader.spec().sample_rate;
    let channels = input_reader.spec().channels;
    println!("Configuration loaded.");
    println!("Sample rate: {sample_rate} Hz");
    println!("Channels: {channels}");

    println!("Calculating slices...");
    let slice_duration = (config.end_time - config.start_time) / 88.0;
    let slice_samples = (slice_duration * sample_rate as f32).floor() as usize;
    let mut slice_indexes: Vec<usize> = Vec::with_capacity(88);
    for i in 0..88 {
        let slice_start_time = config.start_time + (i as f32 * slice_duration);
        let slice_start_sample = (slice_start_time * sample_rate as f32).round() as usize;
        slice_indexes.push(slice_start_sample);
    }
    println!("Slices calculated.");
    println!("Duration per slice: {slice_duration} s");
    println!("Samples per slice: {slice_samples}");

    println!("Analyzing loudness...");
    let samples = input_reader
        .into_samples::<f32>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|msg| msg.to_string())
        .unwrap_or_else(|msg| abort(&msg));
    let split_samples = utils::split_channels(samples, channels);
    let loudness = utils::calculate_loudness(split_samples, slice_indexes, slice_samples);
    println!("Loudness calculated.");

    println!("Calculating volume compensation...");
    let average = utils::moving_average(config.smooth_span, &loudness);
    let compensation = utils::calculate_compensation(loudness, average);
    println!("Volume compensation calculated.");

    println!("Writing output file...");
    utils::write_output_file(config, compensation).unwrap_or_else(|msg| abort(&msg));
    println!("Output file written.");

    println!("Done!\nPress enter to exit.");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}

fn abort(msg: &str) -> ! {
    println!("Error: {msg}\nPress enter to exit.");
    std::io::stdin().read_line(&mut String::new()).unwrap();
    std::process::exit(1);
}
