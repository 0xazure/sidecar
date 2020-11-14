use crate::Post;
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use xml::reader::{EventReader, XmlEvent};

#[derive(PartialEq)]
enum XmlTag {
    Post,
    Tag,
    PhotoUrl,
    Other,
}

pub fn parse_posts<P: AsRef<Path>>(posts_file: P) -> Result<Vec<Post>> {
    let file = File::open(posts_file.as_ref()).context(format!(
        "unable to open posts.xml at {:?}",
        posts_file.as_ref()
    ))?;
    let file = BufReader::new(file);
    let parser = EventReader::new(file);

    let mut posts: Vec<Post> = Vec::new();

    let mut post: Post = Default::default();
    let mut last_opened_tag = XmlTag::Other;

    for event in parser {
        match event {
            Err(e) => return Err(e.into()),
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => match name.local_name.as_str() {
                "post" => {
                    last_opened_tag = XmlTag::Post;
                    post.id = match attributes.iter().find(|a| a.name.local_name == "id") {
                        Some(id) => id.value.clone(),
                        None => bail!("Post missing required attribute 'id'"),
                    }
                }
                "tag" => last_opened_tag = XmlTag::Tag,
                "photo-url" => last_opened_tag = XmlTag::PhotoUrl,
                _ => last_opened_tag = XmlTag::Other,
            },
            Ok(XmlEvent::EndElement { name, .. }) => {
                if name.local_name.as_str() == "post" {
                    if post.extension.is_some() {
                        posts.push(post);
                    }
                    post = Default::default();
                }
            }
            Ok(XmlEvent::Characters(chars)) => match last_opened_tag {
                XmlTag::Tag => post.tags.push(chars),
                XmlTag::PhotoUrl => {
                    if post.extension.is_none() {
                        let mut iter = chars.rsplitn(2, '.');
                        let after = iter.next();
                        let before = iter.next();

                        if before.is_some() {
                            post.extension = after.map(String::from);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        };
    }

    Ok(posts)
}
