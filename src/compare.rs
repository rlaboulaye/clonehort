use anyhow::{Context, Ok, Result};
use rayon::ThreadPoolBuilder;
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

fn process_fb(file: &str, indices: &Vec<usize>, windows: &Vec<[u32; 2]>) -> Result<Vec<Vec<bool>>> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()
        .unwrap();

    let f = File::open(file).with_context(|| format!("Failed to read {}", file))?;
    let mut lines = BufReader::new(f).lines().skip(2);

    Ok(vec![vec![]])
}

// fn jointly_process_msp_fb(
//     msp_file: &str,
//     fb_file: &str,
//     indices: &Vec<usize>,
//     threshold: f32,
// ) -> Result<(Vec<Vec<[u8; 2]>>, Vec<String>)> {
//     let index_set: HashSet<usize> = indices.iter().cloned().collect();
//     let index_max = indices.iter().max().unwrap();
//     let msp_f = File::open(msp_file).with_context(|| format!("Failed to read {}", msp_file))?;
//     let fb_f = File::open(fb_file).with_context(|| format!("Failed to read {}", fb_file))?;
//     let mut msp_lines = BufReader::new(msp_f).lines();
//     let n_label_types = msp_lines
//         .next()
//         .unwrap()
//         .context("Empty file")?
//         .trim()
//         .split(':')
//         .nth(1)
//         .unwrap()
//         .trim()
//         .split('\t')
//         .count();
//     let indexed_samples: Vec<String> = msp_lines
//         .next()
//         .unwrap()
//         .context("Empty file")?
//         .trim()
//         .split('\t')
//         .enumerate()
//         .filter(|(i, _)| index_set.contains(i))
//         .map(|(_, sample)| sample.to_string())
//         .collect();
//     let fb_lines = BufReader::new(fb_f).lines().skip(2);
//     let labels: Vec<Vec<[u8; 2]>> = msp_lines
//         .zip(fb_lines)
//         .map(|(msp_line, fb_line)| {
//             let msp_line = msp_line.unwrap();
//             let fb_line = fb_line.unwrap();
//             // Skip columns that prepend the data
//             let split_fb_line: Vec<&str> = fb_line.trim().split('\t').skip(4).collect();
//             msp_line
//                 .trim()
//                 .split('\t')
//                 .skip(6)
//                 .enumerate()
//                 .take_while(|(i, _)| i <= index_max)
//                 .filter(|(i, _)| index_set.contains(i))
//                 .map(|(i, label)| match label.parse::<u8>() {
//                     Ok(label) => {
//                         return [
//                             if split_fb_line[i * n_label_types + label as usize]
//                                 .parse::<f32>()
//                                 .unwrap_or(-1f32)
//                                 >= threshold
//                             {
//                                 1u8
//                             } else {
//                                 0u8
//                             },
//                             label,
//                         ];
//                     }
//                     Err(_) => return [0u8, 0u8],
//                 })
//                 .collect()
//         })
//         .collect();
//     Ok((labels, indexed_samples))
// }

// fn process_fb(
//     file: &str,
//     indices: &Vec<usize>,
//     labels: &Vec<Vec<u8>>,
//     threshold: f32,
// ) -> Result<Vec<Vec<bool>>> {
//     let f = File::open(file).with_context(|| format!("Failed to read {}", file))?;
//     let mut lines = BufReader::new(f).lines();
//     let n_label_types = lines
//         .next()
//         .unwrap()
//         .context("Empty file")?
//         .trim()
//         .split('\t')
//         .count()
//         - 1;
//     fn to_fb_index(index: usize, label: u8, n_label_types: usize) -> usize {
//         index * n_label_types + label as usize
//     }
//     let index_set: HashSet<usize> = indices.iter().cloned().collect();
//     let last_label = n_label_types - 1;
//     // let fb_index_max = to_fb_index(*indices.iter().max().unwrap(), last_label as u8, n_label_types);
//     let fb: Vec<Vec<bool>> = lines
//         .skip(1)
//         .zip(labels.iter())
//         .map(|(line, label_row)| {
//             line.unwrap()
//                 .trim()
//                 .split('\t')
//                 // Skip columns that prepend data
//                 .skip(4)
//                 .zip(label_row.iter())
//                 .enumerate()
//                 // .take_while(|(i, _)| *i <= fb_index_max)
//                 .filter(|(i, (_, label))| {
//                     println!("{} {} {}", i, label, (i - **label as usize));
//                     (i + last_label - **label as usize) % n_label_types == 0
//                         && index_set
//                             .contains(&((i + last_label - **label as usize) / n_label_types))
//                 })
//                 .map(|(_, (val, _))| val.parse::<f32>().unwrap_or(-1f32) >= threshold)
//                 .collect()
//         })
//         .collect();
//     Ok(fb)
// }

/// Compare the local ancestry inference results for two populations, a reference and a target.
/// Requires the following files: <samples>, <reference>.msp.tsv, <target>.msp.tsv, <reference>.fb.tsv, <target>.fb.tsv.
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

    let (ref_labels, ref_indexed_samples) =
        jointly_process_msp_fb(&ref_msp, &ref_fb, &ref_indices, threshold.unwrap_or(0f32))?;
    let (_, target_labels, target_indexed_samples) = process_msp(&target_msp, &target_indices)?;
    // let ref_filter = process_fb(
    //     &ref_fb,
    //     &ref_indices,
    //     &ref_labels,
    //     threshold.unwrap_or(0f32),
    // )?;

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

    // //
    // println!(
    //     "ref pos: {}",
    //     ref_indexed_samples
    //         .iter()
    //         .position(|s| s == "HG01565.1")
    //         .unwrap()
    // );
    // println!(
    //     "target pos: {}",
    //     index_map[ref_indexed_samples
    //         .iter()
    //         .position(|s| s == "HG01565.1")
    //         .unwrap()]
    // );
    // println!("{}", ref_labels[0][10][0]);
    // println!("{}", ref_labels[0][10][1]);
    // println!("{}", target_labels[0][6]);
    // //

    for (ref_row, target_row) in ref_labels.into_iter().zip(target_labels.into_iter()) {
        for (i, &j) in (0..sample_set.len()).zip(index_map.iter()) {
            if ref_row[i][0] == 1 {
                n_col_total[i] += 1;
                if ref_row[i][1] == target_row[j] {
                    n_col_shared[i] += 1;
                }
            }
        }
    }

    // for ((ref_row, target_row), filter_row) in ref_labels
    //     .into_iter()
    //     .zip(target_labels.into_iter())
    //     .zip(ref_filter.into_iter())
    // {
    //     for (i, &j) in (0..sample_set.len()).zip(index_map.iter()) {
    //         if filter_row[i] {
    //             n_col_total[i] += 1;
    //             if ref_row[i] == target_row[j] {
    //                 n_col_shared[i] += 1;
    //             }
    //         }
    //     }
    // }

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
