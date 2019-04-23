use std::error;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "sidecar",
    about = "Generate sidecar files from Tumblr posts.xml files"
)]
pub struct Config {
    #[structopt(
        name = "posts.xml",
        short = "p",
        long = "posts",
        default_value = "posts.xml"
    )]
    posts_file: PathBuf,
    #[structopt(name = "media/", short = "m", long = "media", default_value = "media/")]
    media_dir: PathBuf,
}

pub fn run(config: Config) -> Result<(), Box<dyn error::Error>> {
    println!("{:?}", config);
    config.posts_file.canonicalize()?;
    config.media_dir.canonicalize()?;

    Ok(())
}
