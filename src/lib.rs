use std::error;
use std::io;
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

impl Config {
    fn exists(&self) -> Result<(), io::Error> {
        if !self.posts_file.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("No such file or directory {}", self.posts_file.display()),
            ));
        }

        if !self.media_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("No such file or directory {}", self.posts_file.display()),
            ));
        }

        Ok(())
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn error::Error>> {
    config.exists()?;

    Ok(())
}
