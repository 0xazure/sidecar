use std::error;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use xml::reader::{EventReader, XmlEvent};

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

#[derive(PartialEq)]
enum XmlTag {
    Post,
    Tag,
    PhotoUrl,
    Other,
}

#[derive(PartialEq, Debug, Default)]
struct Post {
    id: String,
    extension: Option<String>,
    tags: Vec<String>,
}

pub fn run(config: Config) -> Result<(), Box<dyn error::Error>> {
    config.exists()?;

    let file = File::open(config.posts_file)?;
    let file = BufReader::new(file);
    let parser = EventReader::new(file);

    let mut posts: Vec<Post> = Vec::new();

    let mut post: Post = Default::default();
    let mut last_opened_tag = XmlTag::Other;

    for event in parser {
        match event {
            Err(e) => return Err(Box::new(e)),
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => match name.local_name.as_str() {
                "post" => {
                    last_opened_tag = XmlTag::Post;
                    post.id = match attributes.iter().find(|a| a.name.local_name == "id") {
                        Some(id) => id.value.clone(),
                        None => {
                            return Err(Box::new(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Post missing required attribute 'id'",
                            )))
                        }
                    }
                }
                "tag" => last_opened_tag = XmlTag::Tag,
                "photo-url" => last_opened_tag = XmlTag::PhotoUrl,
                _ => last_opened_tag = XmlTag::Other,
            },
            Ok(XmlEvent::EndElement { name, .. }) => match name.local_name.as_str() {
                "post" => {
                    posts.push(post);
                    post = Default::default();
                }
                _ => {}
            },
            Ok(XmlEvent::Characters(chars)) => match last_opened_tag {
                XmlTag::Tag => post.tags.push(chars),
                XmlTag::PhotoUrl => {
                    let mut iter = chars.rsplitn(2, '.');
                    let after = iter.next();
                    let before = iter.next();

                    if before.is_some() {
                        post.extension = after.map(String::from);
                    }
                }
                _ => {}
            },
            _ => {}
        };
    }

    for post in posts {
        println!("{:?}", post);
    }

    Ok(())
}
