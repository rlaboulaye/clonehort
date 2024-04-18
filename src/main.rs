use clonehort::read;

fn main() {
    let path = String::from("/media/storage/1000_genomes/GRCh38/variants/chr20/mxl.chr20.chunk1.GRCh38.vcf.gz");
    //let path = String::from("/media/storage/1000_genomes/GRCh38/variants/chr20/mxl.chr20.GRCh38.vcf.gz");
    read(&path);
}
