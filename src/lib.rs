use anyhow::{bail, Context, Result};
use counter::{Counter, TagCount};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

mod counter;
mod parser;

#[derive(StructOpt, Debug)]
pub struct CommonOpts {
    #[structopt(
        name = "posts.xml",
        short = "p",
        long = "posts",
        default_value = "posts.xml"
    )]
    posts_file: PathBuf,
    #[structopt(name = "tag-mappings", long = "tag-mappings")]
    tag_mapping_file: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "sidecar",
    about = "Generate sidecar files from Tumblr posts.xml files"
)]
pub enum Config {
    #[structopt(name = "generate")]
    Generate {
        #[structopt(name = "media", short = "m", long = "media", default_value = "media")]
        media_dir: PathBuf,
        #[structopt(flatten)]
        common_opts: CommonOpts,
    },
    #[structopt(name = "analyze")]
    Analyze {
        #[structopt(flatten)]
        common_opts: CommonOpts,
    },
}

#[derive(Debug, Default)]
pub struct Post {
    id: String,
    tags: Vec<String>,
}

pub fn run(config: Config) -> Result<()> {
    match config {
        Config::Generate {
            media_dir,
            common_opts,
        } => {
            let posts = parse_posts(common_opts)?;
            write_sidecar_files(&posts, media_dir)?;
        }
        Config::Analyze { common_opts } => {
            let posts = parse_posts(common_opts)?;
            let mut tag_counts = count_tags(&posts);

            tag_counts.sort();

            for t in &tag_counts {
                println!("{}", t);
            }
        }
    };

    Ok(())
}

fn parse_posts(common_opts: CommonOpts) -> Result<Vec<Post>> {
    let CommonOpts {
        posts_file,
        tag_mapping_file,
    } = common_opts;

    let tag_mappings = match tag_mapping_file {
        Some(f) => load_tag_mappings(f)?,
        None => HashMap::new(),
    };

    parser::parse_posts(posts_file, &tag_mappings)
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
            "unable to open media directory {}",
            media_dir.as_ref().display()
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

fn load_tag_mappings<P: AsRef<Path>>(mapping_file: P) -> Result<HashMap<String, Option<String>>> {
    let file = File::open(&mapping_file)?;
    let file = BufReader::new(file);

    let mut mappings = HashMap::new();

    for line in file.lines() {
        match line {
            Ok(tag_mappings) => {
                let parts: Vec<&str> = tag_mappings.split(",").map(|t| t.trim()).collect();

                if parts.len() != 2 {
                    bail!(
                        "format error in tags remapping file {}",
                        &mapping_file.as_ref().display()
                    );
                }

                let source_tag = parts.get(0).unwrap_or(&"").to_string();
                let dest_tag = *parts.get(1).unwrap_or(&"");
                let dest_tag = if dest_tag.is_empty() {
                    None
                } else {
                    Some(String::from(dest_tag))
                };

                if !source_tag.is_empty() {
                    mappings.insert(source_tag, dest_tag);
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(mappings)
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
