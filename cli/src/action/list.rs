use std::io;

use anyhow::Result;
use clap::ArgMatches;
use text_trees::{FormatCharacters, StringTreeNode, TreeFormatting};
use thiserror::Error;

use prs_lib::{store::SecretIterConfig, Secret, Store};

use crate::cmd::matcher::{list::ListMatcher, Matcher};

/// List secrets action.
pub struct List<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> List<'a> {
    /// Construct a new list action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the list action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_list = ListMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_list.store()).map_err(Err::Store)?;

        // List aliases based on filters, sort the list
        let config = SecretIterConfig {
            find_files: !matcher_list.only_aliases(),
            find_symlink_files: !matcher_list.only_non_aliases(),
        };
        let mut secrets: Vec<Secret> = store
            .secret_iter_config(config)
            .filter_name(matcher_list.query())
            .collect();
        secrets.sort_unstable_by(|a, b| a.name.cmp(&b.name));

        // Return nothing if we have an empty list
        if secrets.is_empty() {
            return Ok(());
        }

        // Show a list or tree
        if matcher_list.list() {
            secrets.iter().for_each(|s| println!("{}", s.name));
        } else {
            display_tree(&secrets);
        }

        Ok(())
    }
}

/// Display a secrets tree.
fn display_tree(secrets: &[Secret]) {
    // Build tree nodes from secrets list
    let names: Vec<_> = secrets.iter().map(|s| s.name.as_str()).collect();
    let nodes = tree_nodes("", &names);

    // Build root tree, print to stdout
    StringTreeNode::with_child_nodes(".".into(), nodes.into_iter())
        .write_with_format(
            &mut io::stdout(),
            &TreeFormatting::dir_tree(FormatCharacters::box_chars()),
        )
        .expect("failed to print tree list");
}

/// Build tree nodes from given secret names.
///
/// The prefix defines the prefix to ignore from secret names. Should be `""` when parsing a new
/// tree.
///
/// # Warnings
///
/// The given list must be sorted.
fn tree_nodes(prefix: &str, mut secrets: &[&str]) -> Vec<StringTreeNode> {
    let mut nodes = vec![];

    // Walk through secret names, build list of tree nodes
    while !secrets.is_empty() {
        // Find name of a child node, return if we don't have child
        let child_name = secrets[0]
            .trim_start_matches(prefix)
            .trim_start_matches('/')
            .split('/')
            .next()
            .unwrap();
        if child_name.trim().is_empty() {
            return vec![];
        }

        // Build new prefix including child node
        let child_prefix = if prefix.is_empty() {
            child_name.to_string()
        } else {
            format!("{}/{}", prefix, child_name)
        };

        // Find position after last child having selected name
        let next_child_name = secrets[1..]
            .iter()
            .position(|s| {
                s.trim_start_matches(prefix)
                    .trim_start_matches('/')
                    .split('/')
                    .next()
                    .unwrap()
                    != child_name
            })
            .unwrap_or(secrets.len() - 1)
            + 1;

        // Take children with same name from list, build child node
        let (children, todo) = secrets.split_at(next_child_name);
        secrets = todo;
        nodes.push(StringTreeNode::with_child_nodes(
            child_name.into(),
            tree_nodes(&child_prefix, children).into_iter(),
        ));
    }

    nodes
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),
}
