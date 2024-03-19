use std::env;

use futures::TryStreamExt;
use noodles::vcf;
use tokio::{
    fs::File,
    io::{self, BufReader},
};

pub async fn read(path: &str) -> Result<(), Box<dyn std::error::Error>> {

    let mut reader = File::open(&path)
        .await
        .map(BufReader::new)
        .map(vcf::AsyncReader::new)?;

    let header = reader.read_header().await?;

    let mut records = reader.records(&header);

    while let Some(record) = records.try_next().await? {
//        println!("{:?}", record);
        println!("in while");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = read("/media/storage/1000_genomes/GRCh38/variants/ALL.chr1.shapeit2_integrated_snvindels_v2a_27022019.GRCh38.phased.vcf.gz");
        assert_eq!(result, Ok(()));
    }
}
