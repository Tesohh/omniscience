use camino::{Utf8Path, Utf8PathBuf};

use crate::{
    build::shallow::{Frontmatter, ShallowError},
    config::{self, Config},
    format::{src_to_build_path, typst},
    link, node,
};

pub fn shallow_typst(
    root: impl AsRef<Utf8Path>,
    my_path_canon: &Utf8PathBuf,
    config: &Config,
    nodes: &mut node::Db,
    links: &mut link::Db,
    file: &node::File,
    compile: bool,
) -> Result<(), ShallowError> {
    // query the file to ask for omni-frontmatter
    let frontmatter: Frontmatter = typst::query(
        &root,
        my_path_canon,
        "<omni-frontmatter>",
        &typst::QueryParams {
            format: typst::Format::Html,
            silent: true,
            one: true,
            field: Some("value"),
        },
    )
    .map_err(|err| match err {
        typst::QueryError::TypstError(_, ref message)
            if message == "error: expected exactly one element, found 0\n" =>
        {
            ShallowError::MissingFrontmatter
        }
        _ => err.into(),
    })?;

    // WARN: this assumes that paths in build/nodes.toml are already canonical and valid
    let maybe_node = nodes
        .nodes
        .iter_mut()
        .find(|node| &node.path == my_path_canon);

    // update node, and get my id while i'm at it
    let my_id = match maybe_node {
        Some(node) => {
            node.title = frontmatter.title;
            node.names = frontmatter.names;
            node.tags = frontmatter.tags;

            &node.id
        }
        None => {
            nodes.nodes.push(node::Node {
                id: file.id.clone(),
                path: my_path_canon.clone(),
                kind: node::NodeKind::File,
                title: frontmatter.title,
                names: frontmatter.names,
                tags: frontmatter.tags,
            });

            &file.id
        }
    };

    // query the file to ask for omni-links
    let new_links: Vec<super::shallow::Link> = typst::query(
        &root,
        my_path_canon,
        "<omni-link>",
        &typst::QueryParams {
            format: typst::Format::Html,
            silent: true,
            one: false,
            field: Some("value"),
        },
    )?;

    // remove all links from my_id
    links.links.retain(|l| &l.from != my_id);

    // add new links
    let new_links = new_links.iter().filter_map(|l| {
        let to = match l.ghost {
            false => link::To::Id(l.to.clone().into()),
            true => {
                let filepart = link::FilePart::from_typst_style(&l.to);
                link::To::Ghost(filepart?)
            }
        };

        Some(link::Link {
            from: my_id.clone(),
            to,
            location: None, // TODO:
            alias: None,    // TODO:
        })
    });

    links.links.extend(new_links);

    // compile to html and pdf
    if compile {
        let out_html = src_to_build_path(&root, my_path_canon, "html")
            .expect("both paths should be canonical");

        let mut out_pdf = out_html.clone();
        out_pdf.set_extension("pdf");

        if let Some(parent) = out_html.parent()
            && !std::fs::exists(parent)?
        {
            std::fs::create_dir_all(parent)?;
        }

        match config.typst.output_format {
            config::TypstOutputFormat::Html => {
                typst::compile(&root, my_path_canon, out_html, typst::Format::Html, true)?;
            }
            config::TypstOutputFormat::Pdf => {
                typst::compile(&root, my_path_canon, out_pdf, typst::Format::Pdf, true)?;
            }
            config::TypstOutputFormat::HtmlAndPdf => {
                typst::compile(&root, my_path_canon, out_html, typst::Format::Html, true)?;
                typst::compile(&root, my_path_canon, out_pdf, typst::Format::Pdf, true)?;
            }
        };
    };

    Ok(())
}
