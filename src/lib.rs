use counter::{Counter, TagCount};
use std::error;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

mod counter;
mod parser;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "sidecar",
    about = "Generate sidecar files from Tumblr posts.xml files"
)]
pub enum Config {
    #[structopt(name = "generate", visible_alias = "gen")]
    Generate {
        #[structopt(
            name = "posts.xml",
            short = "p",
            long = "posts",
            default_value = "posts.xml"
        )]
        posts_file: PathBuf,
        #[structopt(name = "media/", short = "m", long = "media", default_value = "media/")]
        media_dir: PathBuf,
    },
    #[structopt(name = "analyze", alias = "analyse")]
    Analyze {
        #[structopt(name = "posts.xml", default_value = "posts.xml")]
        posts_file: PathBuf,
    },
}

impl Config {
    fn exists(&self) -> Result<(), io::Error> {
        match self {
            Config::Generate {
                posts_file,
                media_dir,
            } => {
                if !posts_file.exists() {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("No such file or directory {}", posts_file.display()),
                    ));
                }

                if !media_dir.exists() {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("No such file or directory {}", media_dir.display()),
                    ));
                }
            }
            Config::Analyze { posts_file } => {
                if !posts_file.exists() {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("No such file or directory {}", posts_file.display()),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Post {
    id: String,
    extension: Option<String>,
    tags: Vec<String>,
    image_count: u8,
}

pub fn run(config: Config) -> Result<(), Box<dyn error::Error>> {
    config.exists()?;

    match config {
        Config::Generate {
            posts_file,
            media_dir,
        } => {
            let posts = parser::parse_posts(posts_file)?;
            generate_sidecar_files(&posts, media_dir)?;
        }
        Config::Analyze { posts_file } => {
            let posts = parser::parse_posts(posts_file)?;
            let mut tag_counts = count_tags(&posts);

            tag_counts.sort();

            for t in &tag_counts {
                println!("{}", t);
            }
        }
    };

    Ok(())
}

fn generate_sidecar_files<P: AsRef<Path>>(
    posts: &Vec<Post>,
    media_dir: P,
) -> Result<(), io::Error> {
    for post in posts {
        if post.extension.is_some() {
            let mut path = PathBuf::new();
            path.push(media_dir.as_ref().clone());

            let mut buff = Vec::new();
            for tag in &post.tags {
                writeln!(&mut buff, "{}", tag)?;
            }

            if post.image_count == 0 {
                path.push(&post.id);
                path.set_extension(format!("{}.txt", post.extension.as_ref().unwrap()));

                let mut tags_file = File::create(path)?;
                tags_file.write(&buff)?;
            } else {
                for i in 0..post.image_count {
                    let mut photo_path = path.clone();
                    photo_path.push(format!("{}_{}", post.id, i));
                    photo_path.set_extension(format!("{}.txt", post.extension.clone().unwrap()));

                    let mut tags_file = File::create(photo_path)?;
                    tags_file.write(&buff)?;
                }
            }
        }
    }

    Ok(())
}

fn count_tags(posts: &Vec<Post>) -> Vec<TagCount<&str>> {
    let mut counter = Counter::new();

    for post in posts {
        for tag in &post.tags {
            counter.increment(tag);
        }
    }

    counter.into()
}
