use std::{fs, path::PathBuf, str::FromStr};

use tui_tree_widget::{flatten, TreeItem, TreeState};

use crate::app::App;

pub struct StatefulTree<'a> {
    pub state: TreeState,
    pub items: Vec<TreeItem<'a>>,
}

impl<'a> StatefulTree<'a> {
    pub fn new() -> Self {
        Self {
            state: TreeState::default(),
            items: Vec::new(),
        }
    }

    pub fn next_file(&mut self) {
        let visible = flatten(&self.state.get_all_opened(), &self.items);
        let current_identifier = self.state.selected();
        let current_index = visible
            .iter()
            .position(|o| o.identifier == current_identifier);
        let new_index = current_index.map_or(0, |current_index| {
            current_index.saturating_add(1).min(visible.len() - 1)
        });
        let new_identifier = visible[new_index].identifier.clone();
        self.state.select(new_identifier);
    }

    pub fn previous_file(&mut self) {
        let visible = flatten(&self.state.get_all_opened(), &self.items);
        let current_identifier = self.state.selected();
        let current_index = visible
            .iter()
            .position(|o| o.identifier == current_identifier);
        let new_index = current_index.map_or(0, |current_index| {
            current_index.saturating_sub(1).min(visible.len() - 1)
        });
        let new_identifier = visible[new_index].identifier.clone();
        self.state.select(new_identifier);
    }

    pub fn toggle_collapse(&mut self) {
        self.state.toggle();
    }
}

pub fn make_tree(
    path: PathBuf,
    app: &App,
    files_done: &mut Vec<PathBuf>,
) -> Vec<TreeItem<'static>> {
    let mut files_in_dir = Vec::new();
    let readdir = fs::read_dir(&path);
    if readdir.is_err() {
        return vec![TreeItem::new_leaf(
            path.file_name().unwrap().to_string_lossy().to_string(),
        )];
    }

    let mut paths: Vec<_> = readdir.unwrap().map(|r| r.unwrap()).collect();
    paths.sort_by_key(|f| f.path());
    paths.sort_by_key(|f| !f.path().is_dir());

    'outer: for file in paths {
        for done in &*files_done {
            if file.path().starts_with(done) {
                continue 'outer;
            }
        }

        let mut is_torrent_file = false;
        for torrent_file in app.get_selected_torrent().files.as_ref().unwrap() {
            let torrent_file_path = app
                .get_selected_torrent()
                .download_dir
                .as_deref()
                .unwrap()
                .to_owned();
            let torrent_file_path = torrent_file_path + &torrent_file.name;
            let torrent_file_path = PathBuf::from_str(&torrent_file_path).unwrap();

            if file.path().starts_with(&torrent_file_path)
                || torrent_file_path.starts_with(file.path())
            {
                is_torrent_file = true;
                break;
            }
        }

        if !is_torrent_file {
            files_done.push(file.path());
            continue;
        }

        let item = if file.path().is_dir() {
            let children = make_tree(file.path(), app, files_done);
            TreeItem::new(
                file.path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                children,
            )
        } else {
            TreeItem::new_leaf(
                file.path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            )
        };

        files_in_dir.push(item);
    }

    files_in_dir
}
