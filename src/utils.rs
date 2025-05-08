use crate::config::Config;
use rayon::prelude::*;
use std::io::Write;

pub(crate) fn split_channels(samples: Vec<f32>, channels: u16) -> Vec<Vec<f32>> {
    let channels = channels as usize;
    let len = samples.len() / channels;
    let mut result = vec![Vec::with_capacity(len); channels];
    for ch in 0..channels {
        result[ch].extend(samples[ch..].iter().step_by(channels).copied());
    }
    result
}

pub(crate) fn calculate_loudness(
    split_samples: Vec<Vec<f32>>,
    slice_indexes: Vec<usize>,
    slice_samples: usize,
) -> Vec<f32> {
    let channels = split_samples.len();
    let keys = slice_indexes.len();
    let mut loudness = vec![Vec::with_capacity(channels); keys];
    loudness
        .par_iter_mut()
        .enumerate()
        .for_each(|(key, ch_vec)| {
            split_samples.iter().for_each(|samples| {
                let index = slice_indexes[key];
                let rms = rms_loudness(&samples[index..index + slice_samples]);
                ch_vec.push(rms);
            });
        });
    loudness
        .iter()
        .map(|ch_vec| ch_vec.iter().sum::<f32>() / channels as f32)
        .collect()
}

fn rms_loudness(samples: &[f32]) -> f32 {
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    let rms = (sum_sq / samples.len() as f32).sqrt();
    20.0 * rms.log10()
}

pub(crate) fn moving_average(smooth_span: u8, loudness: &Vec<f32>) -> Vec<f32> {
    let smooth_span = smooth_span as usize;
    let mut result = vec![0.0; loudness.len()];
    result.par_iter_mut().enumerate().for_each(|(i, v)| {
        let start = i.saturating_sub(smooth_span).max(0);
        let end = i.saturating_add(smooth_span).min(loudness.len() - 1);
        let slice = &loudness[start..=end];
        *v = slice.into_iter().sum::<f32>() / slice.len() as f32;
    });
    result
}

pub(crate) fn calculate_compensation(loudness: Vec<f32>, average: Vec<f32>) -> Vec<f32> {
    average
        .into_iter()
        .zip(loudness.into_iter())
        .map(|(a, l)| a - l)
        .collect()
}

pub(crate) fn write_output_file(
    config: Config,
    compensation: Vec<f32>,
) -> Result<(), &'static str> {
    let bytes: Vec<u8> = compensation
        .iter()
        .map(|x| x.to_le_bytes())
        .flatten()
        .collect();
    let mut output_bin_file = config
        .get_output_bin_file()
        .map_err(|_| "Failed to open output file")?;
    output_bin_file
        .write_all(&bytes)
        .map_err(|_| "Failed to write to output file")?;
    let string = compensation
        .into_iter()
        .map(|x| format!("{:.9}", x))
        .collect::<Vec<_>>()
        .join("\n");
    let mut output_txt_file = config
        .get_output_txt_file()
        .map_err(|_| "Failed to open output file")?;
    Ok(output_txt_file
        .write_all(string.as_bytes())
        .map_err(|_| "Failed to write to output file")?)
}
