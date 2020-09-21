use clap::Clap;

#[derive(Clap)]
struct Opts {
    #[clap(long)]
    emit_existing: bool,
    #[clap(long)]
    normalize: bool,
}

fn main() {
    let opts = Opts::parse();
}
