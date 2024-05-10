use anyhow::{Context, Ok, Result};
use std::collections::HashSet;
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::iter::zip;
use std::sync::{Arc, Mutex};

fn first_line_id_indices(file: &str, sample_set: &HashSet<String>) -> Result<Vec<usize>> {
    let f = File::open(file).with_context(|| format!("Failed to read {}", file))?;
    let indices: Vec<usize> = BufReader::new(f)
        .lines()
        .nth(1)
        .unwrap()
        .context("Empty file")?
        .trim()
        .split('\t')
        // Skip columns that prepend data
        .skip(6)
        .enumerate()
        .filter(|(_, sample)| sample_set.contains(*sample))
        .map(|(i, _)| i)
        .collect();
    Ok(indices)
}

fn process_msp(
    file: &str,
    indices: &Vec<usize>,
) -> Result<(Vec<[u32; 2]>, Vec<Vec<u8>>, Vec<String>)> {
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
        // Skip columns that prepend data
        .skip(6)
        .enumerate()
        .filter(|(i, _)| index_set.contains(i))
        .map(|(_, sample)| sample.to_string())
        .collect();
    let (windows, labels): (Vec<[u32; 2]>, Vec<Vec<u8>>) = lines
        .map(|line| {
            let line = line.unwrap();
            let mut split_line = line.trim().split('\t').skip(1);
            let window = [
                split_line.next().unwrap().parse::<u32>().unwrap(),
                split_line.next().unwrap().parse::<u32>().unwrap(),
            ];
            (
                window,
                split_line
                    // Skip columns that prepend data
                    .skip(3)
                    .enumerate()
                    .take_while(|(i, _)| i <= index_max)
                    .filter(|(i, _)| index_set.contains(i))
                    .map(|(_, val)| val.parse::<u8>().unwrap_or(u8::MAX))
                    .collect(),
            )
        })
        .unzip();
    Ok((windows, labels, indexed_samples))
}

fn process_fb(
    file: &str,
    indices: &Vec<usize>,
    windows: &Vec<[u32; 2]>,
    labels: &Vec<Vec<u8>>,
    threshold: Option<f32>,
) -> Result<Vec<Vec<bool>>> {
    let mut filter: Arc<Mutex<Vec<Vec<bool>>>> =
        Arc::new(Mutex::new(vec![vec![true; indices.len()]; windows.len()]));

    let index_set: HashSet<usize> = indices.iter().cloned().collect();

    let f = File::open(file).with_context(|| format!("Failed to read {}", file))?;
    let mut lines = BufReader::new(f).lines();

    let n_label_types = lines
        .next()
        .unwrap()
        .context("Empty file")?
        .trim()
        .split('\t')
        .count()
        - 1;

    rayon::scope(|scope| {
        let mut window_counter: usize = 0;
        let mut line_block: Vec<Box<dyn Iterator<Item = &str> + Send>> = vec![];
        let line = lines.skip(1).next();

        while let Some(line) = line {
            let line = line.unwrap();
            let mut split_line = line.trim().split('\t').skip(1);
            let pos = split_line.next().unwrap().parse::<u32>().unwrap();
            if pos > windows[window_counter][1] {
                scope.spawn(|_| {
                    let row_index = window_counter;
                    let line_block = line_block;
                    let (prob_sums, row_count) = line_block
                        .into_iter()
                        .map(|line| {
                            line.enumerate()
                                .filter(|(i, _)| {
                                    index_set.contains(&(i / n_label_types))
                                        && i % n_label_types
                                            == labels[row_index][i / n_label_types] as usize
                                })
                                .map(|(_, val)| val.parse::<f32>().unwrap_or(f32::MIN))
                                .collect::<Vec<f32>>()
                        })
                        .fold((Vec::<f32>::new(), 0), |(prob_sums, row_count), probs| {
                            (
                                probs
                                    .iter()
                                    .zip(prob_sums.iter())
                                    .map(|(p, s)| p + s)
                                    .collect(),
                                row_count + 1,
                            )
                        });
                    let mut filter_matrix = filter.lock().unwrap();
                    for (i, prob_sum) in prob_sums.iter().enumerate() {
                        if *prob_sum / row_count as f32 >= threshold.unwrap_or(0f32) {
                            filter_matrix[row_index][i] = true;
                        } else {
                            filter_matrix[row_index][i] = false;
                        }
                    }
                });
                line_block = vec![];
                line_block.push(Box::new(split_line.skip(2)));
                window_counter += 1;
                if window_counter == windows.len() {
                    break;
                }
            } else if pos >= windows[window_counter][0] {
                line_block.push(Box::new(split_line.skip(2)));
            }
            let line = lines.next();
        }
    });

    Ok(filter.into_inner().unwrap())
}

/// Compare the local ancestry inference results for two populations, a reference and a target.
/// Requires the following files: <samples>, <reference>.msp.tsv, <target>.msp.tsv, <reference>.fb.tsv.
///
/// # Arguments
///
/// * `samples` - A newline-separated file of sample names to compare.
/// * `reference` - Path and prefix of the reference population.
/// * `target` - Path and prefix of the target population.
/// * `threshold` - Posterior probability threshold for the inclusion of a locus in the comparison.
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

    let (windows, ref_labels, ref_indexed_samples) = process_msp(&ref_msp, &ref_indices)?;
    let (_, target_labels, target_indexed_samples) = process_msp(&target_msp, &target_indices)?;
    let filter = process_fb(&ref_fb, &ref_indices, &windows, &ref_labels, threshold)?;

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
        .zip(filter.into_iter())
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toy_msp() {
        let samples_path = "data/test/toy_samples.txt";
        let ref_path = "data/test/toy_ref.msp.tsv";
        let target_path = "data/test/toy_target.msp.tsv";

        // refactor perform_comparison with threshold and fb optional and return shared

        let sample_set: HashSet<String> = read_to_string(samples_path)
            .with_context(|| format!("Failed to read {}", samples_path))
            .unwrap()
            .trim()
            .split('\n')
            // .map(|s| String::from(s))
            .map(|s| [format!("{}.0", s), format!("{}.1", s)])
            .flatten()
            .collect();

        let ref_indices = first_line_id_indices(ref_path, &sample_set).unwrap();
        let target_indices = first_line_id_indices(target_path, &sample_set).unwrap();

        let (_, ref_labels, ref_indexed_samples) = process_msp(ref_path, &ref_indices).unwrap();
        let (_, target_labels, target_indexed_samples) =
            process_msp(target_path, &target_indices).unwrap();

        let index_map: Vec<usize> = ref_indexed_samples
            .iter()
            .map(|s1| {
                target_indexed_samples
                    .iter()
                    .position(|s2| s1 == s2)
                    .unwrap()
            })
            .collect();

        let mut n_col_shared = vec![0; sample_set.len()];

        for (ref_row, target_row) in ref_labels.into_iter().zip(target_labels.into_iter()) {
            for (i, &j) in (0..sample_set.len()).zip(index_map.iter()) {
                if ref_row[i] == target_row[j] {
                    n_col_shared[i] += 1;
                }
            }
        }

        assert_eq!(n_col_shared[0], 7);
        assert_eq!(n_col_shared[1], 7);
        assert_eq!(n_col_shared[2], 5);
        assert_eq!(n_col_shared[3], 6);
        assert_eq!(n_col_shared[4], 0);
        assert_eq!(n_col_shared[5], 7);
        assert_eq!(n_col_shared[6], 4);
        assert_eq!(n_col_shared[7], 7);
    }
}
