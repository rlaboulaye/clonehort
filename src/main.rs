use clonehort::read;

fn main() {
    let path = String::from("/media/storage/1000_genomes/GRCh38/variants/ALL.chr1.shapeit2_integrated_snvindels_v2a_27022019.GRCh38.phased.vcf.gz");
    read(&path);
}
