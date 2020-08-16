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
    #[structopt(name = "generate")]
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
    #[structopt(name = "analyze")]
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
            write_sidecar_files(&posts, media_dir)?;
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

fn write_sidecar_files<P: AsRef<Path>>(posts: &[Post], media_dir: P) -> Result<(), io::Error> {
    for post in posts {
        let mut path = PathBuf::new();
        path.push(media_dir.as_ref());

        let mut buff = Vec::with_capacity(post.tags.iter().fold(0, |a, t| a + t.len() + 1));
        for tag in &post.tags {
            writeln!(&mut buff, "{}", tag)?;
        }

        if post.image_count > 0 {
            for i in 0..post.image_count {
                write_sidecar_file(
                    path.clone(),
                    post.id.clone(),
                    Some(i),
                    post.extension.as_ref().unwrap(),
                    &buff,
                )?;
            }
        } else {
            write_sidecar_file(
                path.clone(),
                post.id.clone(),
                None,
                post.extension.as_ref().unwrap(),
                &buff,
            )?;
        }
    }

    Ok(())
}

fn write_sidecar_file<E: AsRef<str>>(
    output_dir: PathBuf,
    image_id: String,
    image_offset: Option<u8>,
    extension: E,
    tags: &[u8],
) -> Result<(), io::Error> {
    let mut file_path = output_dir;
    let mut filename = image_id;

    if image_offset.is_some() {
        filename += &format!("_{}", image_offset.unwrap());
    }

    file_path.push(filename);
    file_path.set_extension(format!("{}.txt", extension.as_ref()));

    let mut tags_file = File::create(file_path)?;
    tags_file.write_all(&tags)?;

    Ok(())
}

fn count_tags(posts: &[Post]) -> Vec<TagCount<&str>> {
    let mut counter: Counter = Default::default();

    for post in posts {
        for tag in &post.tags {
            counter.increment(tag);
        }
    }

    counter.into()
}
