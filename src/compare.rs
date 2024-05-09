use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::iter::zip;

fn first_line_id_indices(file: &str, sample_set: &HashSet<String>) -> Result<Vec<usize>> {
    let f = File::open(file).with_context(|| format!("Failed to read {}", file))?;
    let indices: Vec<usize> = BufReader::new(f)
        .lines()
        .nth(1)
        .unwrap()
        .context("Empty file")?
        .trim()
        .split('\t')
        .enumerate()
        .filter(|(_, sample)| sample_set.contains(*sample))
        .map(|(i, _)| i)
        .collect();
    Ok(indices)
}

fn process_fb(file: &str, indices: &Vec<usize>, threshold: f32) -> Result<Vec<Vec<bool>>> {
    let index_set: HashSet<usize> = indices.iter().cloned().collect();
    let index_max = indices.iter().max().unwrap();
    let f = File::open(file).with_context(|| format!("Failed to read {}", file))?;
    let fb: Vec<Vec<bool>> = BufReader::new(f)
        .lines()
        .skip(2)
        .map(|line| {
            line.unwrap()
                .trim()
                .split('\t')
                .enumerate()
                .take_while(|(i, _)| i <= index_max)
                .filter(|(i, _)| index_set.contains(i))
                .map(|(_, val)| val.parse::<f32>().unwrap_or(-1f32) >= threshold)
                .collect()
        })
        .collect();
    Ok(fb)
}

fn process_msp(file: &str, indices: &Vec<usize>) -> Result<(Vec<Vec<u8>>, Vec<String>)> {
    let index_set: HashSet<usize> = indices.iter().cloned().collect();
    let index_max = indices.iter().max().unwrap();
    let f = File::open(file).with_context(|| format!("Failed to read {}", file))?;
    let mut lines = BufReader::new(f).lines().skip(1);
    let indexed_samples: Vec<String> = lines
        .next()
        .unwrap()
        .context("Empty file")?
        .trim()
        .split('\t')
        .enumerate()
        .filter(|(i, _)| index_set.contains(i))
        .map(|(_, sample)| sample.to_string())
        .collect();
    let msp: Vec<Vec<u8>> = lines
        .map(|line| {
            line.unwrap()
                .trim()
                .split('\t')
                .enumerate()
                .take_while(|(i, _)| i <= index_max)
                .filter(|(i, _)| index_set.contains(i))
                .map(|(_, val)| val.parse::<u8>().unwrap_or(u8::MAX))
                .collect()
        })
        .collect();
    Ok((msp, indexed_samples))
}

/// Compare the local ancestry inference results for two populations, a reference and a target.
/// Requires the following files: <samples>, <reference>.msp.tsv, <target>.msp.tsv, <reference>.fb.tsv, <target>.fb.tsv.
///
/// # Arguments
///
/// * `samples` - A newline-separated file of sample names to compare.
/// * `reference` - Path and prefix of the reference population.
/// * `target` - Path and prefix of the target population.
pub fn perform_comparison(
    samples: &str,
    reference: &str,
    target: &str,
    threshold: Option<f32>,
) -> Result<()> {
    let ref_msp = format!("{}.msp.tsv", reference);
    let target_msp = format!("{}.msp.tsv", target);
    let ref_fb = format!("{}.fb.tsv", reference);

    // Read the samples file
    let sample_set: HashSet<String> = read_to_string(samples)
        .with_context(|| format!("Failed to read {}", samples))?
        .trim()
        .split('\n')
        // .map(|s| String::from(s))
        .map(|s| [format!("{}.0", s), format!("{}.1", s)])
        .flatten()
        .collect();

    let ref_indices = first_line_id_indices(&ref_msp, &sample_set)?;
    let target_indices = first_line_id_indices(&target_msp, &sample_set)?;

    if sample_set.len() != ref_indices.len() || sample_set.len() != target_indices.len() {
        return Err(anyhow::anyhow!(
            "Some sample ids in the samples file are missing from the msp files."
        ));
    }

    let ref_filter = process_fb(&ref_fb, &ref_indices, threshold.unwrap_or(0f32))?;
    let (ref_labels, ref_indexed_samples) = process_msp(&ref_msp, &ref_indices)?;
    let (target_labels, target_indexed_samples) = process_msp(&target_msp, &target_indices)?;

    let index_map: Vec<usize> = ref_indexed_samples
        .iter()
        .map(|s1| {
            target_indexed_samples
                .iter()
                .position(|s2| s1 == s2)
                .unwrap()
        })
        .collect();

    let mut n_total = 0;
    let mut n_shared = 0;
    let mut n_col_total = vec![0; sample_set.len()];
    let mut n_col_shared = vec![0; sample_set.len()];

    for ((ref_row, target_row), filter_row) in ref_labels
        .into_iter()
        .zip(target_labels.into_iter())
        .zip(ref_filter.into_iter())
    {
        for (i, &j) in (0..sample_set.len()).zip(index_map.iter()) {
            if filter_row[i] {
                n_col_total[i] += 1;
                if ref_row[i] == target_row[j] {
                    n_col_shared[i] += 1;
                }
            }
        }
    }

    for (i, (total, shared)) in zip(n_col_total.iter(), n_col_shared.iter()).enumerate() {
        println!(
            "Sample {}: {}/{} = {} shared",
            ref_indexed_samples[i],
            shared,
            total,
            *shared as f32 / *total as f32
        );
        n_total += total;
        n_shared += shared;
    }
    println!(
        "Total: {}/{} = {} shared",
        n_shared,
        n_total,
        n_shared as f32 / n_total as f32
    );

    // for (i, &j) in (0..sample_set.len()).zip(index_map.iter()) {
    //     println!("{}: {}", ref_indexed_samples[i], target_indexed_samples[j]);
    // }

    Ok(())
}
