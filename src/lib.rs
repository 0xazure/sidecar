use anyhow::{Context, Result};
use counter::{Counter, TagCount};
use std::fs::{self, File};
use std::io::Write;
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

#[derive(Debug, Default)]
pub struct Post {
    id: String,
    extension: Option<String>,
    tags: Vec<String>,
}

pub fn run(config: Config) -> Result<()> {
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

fn write_sidecar_files<P: AsRef<Path>>(posts: &[Post], media_dir: P) -> Result<()> {
    // Build a sorted cache of media files on disk to more efficiently generate
    // sidecar files for all files related to a given post instead of relying
    // solely on the photoset data in `posts.xml` to determine suffixes for
    // files in multi-photo posts. Relying only on `posts.xml` leaves out any
    // files added to reblogs of the original post which are also included in
    // the export and should also generate a sidecar file.
    //
    // Note that we do not sort this cache as (based on preliminary testing)
    // later calls to `filter()` to search the cache for files with specific
    // prefixes cannot take advantage of sorting. If we get more clever about
    // cache searching this may change.
    let files: Vec<fs::DirEntry> = fs::read_dir(&media_dir)
        .context(format!(
            "unable to open media directory {:?}",
            media_dir.as_ref()
        ))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.path().extension().map_or(true, |ext| ext != "txt"))
        .collect();

    for post in posts {
        let mut tags = Vec::with_capacity(post.tags.iter().fold(0, |a, t| a + t.len() + 1));
        for tag in &post.tags {
            writeln!(&mut tags, "{}", tag)?;
        }

        for entry in files
            .iter()
            .filter(|e| {
                e.path()
                    .file_stem()
                    .map_or(false, |f| f.to_string_lossy().starts_with(&post.id))
            })
            .collect::<Vec<&fs::DirEntry>>()
        {
            let path = entry.path();
            // Only write sidecar files for source files that actually exist,
            // since the initial file cache can get out of sync.
            if entry.path().exists() {
                let file_path = path.to_string_lossy() + ".txt";
                let mut tags_file = File::create(file_path.as_ref())?;
                tags_file.write_all(&tags)?;
            }
        }
    }

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
