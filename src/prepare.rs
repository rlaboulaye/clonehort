use anyhow::{Ok, Result};
use rust_htslib::bcf::{Read, Reader};
use std::convert::TryFrom;

fn read_vcf(path: &str) -> Result<()> {
    // let path = &"test/test_string.vcf";
    let mut vcf = Reader::from_path(path).expect("Error opening file.");
    // iterate through each row of the vcf body.
    for (_i, record_result) in vcf.records().enumerate() {
        let record = record_result.expect("Fail to read record");
        let mut s = String::new();
        for allele in record.alleles() {
            for c in allele {
                s.push(char::from(*c))
            }
            s.push(' ')
        }
        // 0-based position and the list of alleles
        println!("Locus: {}, Alleles: {}", record.pos(), s);
        // number of sample in the vcf
        let sample_count = usize::try_from(record.sample_count()).unwrap();

        // Counting ref, alt and missing alleles for each sample
        let mut n_ref = vec![0; sample_count];
        let mut n_alt = vec![0; sample_count];
        let mut n_missing = vec![0; sample_count];
        let gts = record.genotypes().expect("Error reading genotypes");
        for sample_index in 0..sample_count {
            // for each sample
            for gta in gts.get(sample_index).iter() {
                // for each allele
                match gta.index() {
                    Some(0) => n_ref[sample_index] += 1,  // reference allele
                    Some(_) => n_alt[sample_index] += 1,  // alt allele
                    None => n_missing[sample_index] += 1, // missing allele
                }
            }
        }
    }
    Ok(())
}

pub fn process_variant_input(vcf_path: &str) -> Result<()> {
    read_vcf(vcf_path)
}
