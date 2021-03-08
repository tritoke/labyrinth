use std::path::PathBuf;

use rand::{rngs::SmallRng, SeedableRng};
use structopt::StructOpt;

mod maze;
use maze::Maze;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Labyrinth",
    about = "Maze generation program.",
    version = "0.0.1",
    author = "Sam L. (@_tritoke)"
)]
struct Opt {
    /// file to save the rendered image to
    #[structopt(short, long = "out", parse(from_os_str), default_value = "maze.png")]
    outfile: PathBuf,

    /// seed for the RNG
    #[structopt(short, long)]
    seed: Option<u64>,

    /// width of the rendered image in pixels
    #[structopt(short, long, default_value = "500")]
    width: u32,

    /// width of the rendered image in pixels
    #[structopt(short, long, default_value = "500")]
    height: u32,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let mut rng = if let Some(seed) = opt.seed {
        SmallRng::seed_from_u64(seed)
    } else {
        SmallRng::from_entropy()
    };

    let mut maze = Maze::new(opt.width, opt.height);
    maze.populate(&mut rng);
    maze.save_to_file(&opt.outfile)?;

    Ok(())
}
