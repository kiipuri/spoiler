use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use chrono::{DateTime, NaiveDateTime, Utc};
use transmission_rpc::{
    types::{
        Id, RpcResponse, SessionGet, Torrent, TorrentAction, TorrentAddArgs, TorrentGetField,
        Torrents,
    },
    TransClient,
};
use tui::widgets::Row;
use walkdir::{DirEntry, WalkDir};

use crate::{
    config::Config,
    conversion::{convert_rate, get_percentage, status_string},
};

pub enum RouteId {
    TorrentList,
    TorrentInfo,
}

pub struct Route {
    pub id: RouteId,
    pub focused_widget: FocusableWidget,
}

pub enum FocusableWidget {
    TorrentList,
    Tabs,
    FileList,
}

pub enum FloatingWidget {
    Help,
    Input,
    AddTorrent,
    AddTorrentConfirm,
    RemoveTorrent,
    ModifyColumns,
    None,
}

pub enum InputMode {
    Normal,
    Editing,
}

pub enum ColumnField {
    Name,
    Status,
    Id,
    Progress,
    Eta,
    DownloadRate,
    UploadRate,
    UploadRatio,
    DoneDate,
    AddedDate,
}

impl ColumnField {
    pub fn to_str(&self) -> String {
        match self {
            ColumnField::Name => "Name",
            ColumnField::Status => "Status",
            ColumnField::Id => "Id",
            ColumnField::Progress => "Progress",
            ColumnField::Eta => "Eta",
            ColumnField::DownloadRate => "Down Speed",
            ColumnField::UploadRate => "Up Speed",
            ColumnField::UploadRatio => "Ratio",
            ColumnField::DoneDate => "Date Done",
            ColumnField::AddedDate => "Date Added",
        }
        .to_string()
    }
}

pub struct CollapsePath {
    pub path: DirEntry,
    pub collapse: bool,
}

pub struct ColumnAndShow {
    pub column: ColumnField,
    pub show: bool,
}

pub struct App {
    pub config: Config,
    pub navigation_stack: Vec<Route>,
    pub torrents: RpcResponse<Torrents<Torrent>>,
    pub selected_torrent: Option<usize>,
    pub selected_tab: usize,
    pub selected_file: Option<usize>,
    pub floating_widget: FloatingWidget,
    pub should_quit: bool,
    pub sort_descending: bool,
    pub sort_column: u32,
    pub input_mode: InputMode,
    pub input: String,
    pub torrent_files: Vec<PathBuf>,
    pub selected_torrent_file: Option<usize>,
    pub add_paused: bool,
    pub delete_files: bool,
    pub all_info_columns: Vec<ColumnAndShow>,
    pub selected_column: Option<usize>,
    pub torrent_collapse_files: Vec<CollapsePath>,
}

impl App {
    pub async fn new() -> Self {
        let client = TransClient::new("http://localhost:9091/transmission/rpc");
        let response: transmission_rpc::types::Result<RpcResponse<SessionGet>> =
            client.session_get().await;
        match response {
            Ok(_) => (),
            Err(_) => panic!("Oh no!"),
        }

        let response = client.torrent_get(None, None).await;
        let mut torrents = response.unwrap();
        torrents.arguments.torrents.sort_by(|a, b| {
            a.name
                .as_ref()
                .unwrap()
                .to_lowercase()
                .cmp(&b.name.as_ref().unwrap().to_lowercase())
        });

        Self {
            config: Config::new(),
            navigation_stack: vec![Route {
                id: RouteId::TorrentList,
                focused_widget: FocusableWidget::TorrentList,
            }],
            floating_widget: FloatingWidget::None,
            selected_torrent: Some(0),
            selected_file: None,
            selected_tab: 0,
            should_quit: false,
            torrents,
            sort_descending: true,
            sort_column: 0,
            input_mode: InputMode::Normal,
            input: String::new(),
            torrent_files: vec![],
            selected_torrent_file: Some(0),
            add_paused: false,
            delete_files: false,
            all_info_columns: vec![
                ColumnAndShow {
                    column: ColumnField::Name,
                    show: true,
                },
                ColumnAndShow {
                    column: ColumnField::Status,
                    show: true,
                },
                ColumnAndShow {
                    column: ColumnField::Progress,
                    show: true,
                },
                ColumnAndShow {
                    column: ColumnField::DownloadRate,
                    show: true,
                },
                ColumnAndShow {
                    column: ColumnField::UploadRate,
                    show: true,
                },
                ColumnAndShow {
                    column: ColumnField::UploadRatio,
                    show: true,
                },
                ColumnAndShow {
                    column: ColumnField::Eta,
                    show: false,
                },
                ColumnAndShow {
                    column: ColumnField::DoneDate,
                    show: false,
                },
                ColumnAndShow {
                    column: ColumnField::AddedDate,
                    show: false,
                },
                ColumnAndShow {
                    column: ColumnField::Id,
                    show: false,
                },
            ],
            selected_column: Some(0),
            torrent_collapse_files: Vec::new(),
        }
    }

    pub fn last_route_id(&self) -> Option<&RouteId> {
        if let Some(i) = self.navigation_stack.last() {
            Some(&i.id)
        } else {
            None
        }
    }

    pub fn last_route_focused_widget(&self) -> Option<&FocusableWidget> {
        if let Some(i) = self.navigation_stack.last() {
            Some(&i.focused_widget)
        } else {
            None
        }
    }

    pub fn next(&mut self) {
        self.selected_torrent =
            Some((self.selected_torrent.unwrap() + 1) % self.torrents.arguments.torrents.len());
        // self.show_tree();
    }

    pub fn previous(&mut self) {
        if self.selected_torrent > Some(0) {
            self.selected_torrent = Some(self.selected_torrent.unwrap() - 1);
        } else {
            self.selected_torrent = Some(self.torrents.arguments.torrents.len() - 1);
        }
        // self.show_tree();
    }

    pub fn next_column(&mut self) {
        self.selected_column = Some((self.selected_column.unwrap() + 1) % 10);
    }

    pub fn previous_column(&mut self) {
        if self.selected_column > Some(0) {
            self.selected_column = Some(self.selected_column.unwrap() - 1);
        } else {
            self.selected_column = Some(9);
        }
    }

    pub fn move_column_down(&mut self) {
        self.all_info_columns.swap(
            self.selected_column.unwrap(),
            (self.selected_column.unwrap() + 1) % 10,
        );
    }

    pub fn move_column_up(&mut self) {
        if self.selected_column == Some(0) {
            self.all_info_columns.swap(self.selected_column.unwrap(), 9);
        } else {
            self.all_info_columns.swap(
                self.selected_column.unwrap() - 1,
                self.selected_column.unwrap(),
            );
        }
    }

    pub fn next_torrent_file(&mut self) {
        self.selected_torrent_file =
            Some((self.selected_torrent_file.unwrap() + 1) % self.torrent_files.len());
    }

    pub fn previous_torrent_file(&mut self) {
        if self.selected_torrent_file > Some(0) {
            self.selected_torrent_file = Some(self.selected_torrent_file.unwrap() - 1);
        } else {
            self.selected_torrent_file = Some(self.torrent_files.len() - 1);
        }
    }

    pub fn stack_push(&mut self, route: Route) {
        self.navigation_stack.push(route);
    }

    pub fn stack_pop(&mut self) {
        self.navigation_stack.pop();
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % 3;
    }

    pub fn previous_tab(&mut self) {
        if self.selected_tab > 0 {
            self.selected_tab -= 1;
        } else {
            self.selected_tab = 3 - 1;
        }
    }

    pub fn next_file(&mut self) {
        self.selected_file = Some(
            (self.selected_file.unwrap() + 1) % self.torrent_collapse_files.len(), // % self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
                                                                                   //     .files
                                                                                   //     .as_ref()
                                                                                   //     .unwrap()
                                                                                   //     .len(),
        );
    }

    pub fn previous_file(&mut self) {
        if self.selected_file.unwrap() > 0 {
            self.selected_file = Some(self.selected_file.unwrap() - 1);
        } else {
            self.selected_file = Some(
                self.torrent_collapse_files.len() - 1, // self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
                                                       //     .files
                                                       //     .as_ref()
                                                       //     .unwrap()
                                                       //     .len()
                                                       //     - 1,
            )
        }
    }

    // pub async fn increment_priority(&mut self) {
    //     self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
    //         .priorities
    //         .as_mut()
    //         .unwrap()[self.selected_file.unwrap()] = 1i8;
    // }

    pub async fn toggle_torrent_pause(&mut self) {
        let client = TransClient::new("http://localhost:9091/transmission/rpc");

        let id = self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
            .id
            .unwrap();

        let status = client
            .torrent_get(Some(vec![TorrentGetField::Status]), Some(vec![Id::Id(id)]))
            .await
            .unwrap()
            .arguments
            .torrents[0]
            .status;

        let mut action = TorrentAction::Stop;
        if status == Some(0) {
            action = TorrentAction::Start;
        }

        client
            .torrent_action(
                action,
                vec![Id::Id(
                    self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
                        .id
                        .unwrap(),
                )],
            )
            .await
            .unwrap();
    }

    pub fn get_torrent_rows(&self) -> (Vec<String>, Vec<Row>) {
        let mut rows = Vec::new();
        for torrent in &self.torrents.arguments.torrents {
            let mut row_strs = Vec::new();
            for field in &self.all_info_columns {
                if !field.show {
                    continue;
                }

                match field.column {
                    ColumnField::Name => {
                        row_strs.push(torrent.name.as_ref().unwrap().to_owned());
                    }
                    ColumnField::Id => {
                        row_strs.push(torrent.id.unwrap().to_string());
                    }
                    ColumnField::Eta => {
                        row_strs.push(torrent.eta.unwrap().to_string());
                    }
                    ColumnField::Status => {
                        row_strs.push(status_string(torrent.status.as_ref().unwrap()).to_string());
                    }
                    ColumnField::Progress => {
                        row_strs.push(get_percentage(
                            torrent.percent_done.as_ref().unwrap().to_owned(),
                        ));
                    }
                    ColumnField::DownloadRate => {
                        row_strs.push(convert_rate(*torrent.rate_download.as_ref().unwrap()));
                    }
                    ColumnField::UploadRate => {
                        row_strs.push(convert_rate(*torrent.rate_upload.as_ref().unwrap()));
                    }
                    ColumnField::UploadRatio => {
                        row_strs.push(format!("{:.2}", torrent.upload_ratio.as_ref().unwrap()));
                    }
                    ColumnField::DoneDate => {
                        let naive = NaiveDateTime::from_timestamp(torrent.done_date.unwrap(), 0);
                        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
                        let newdate = datetime.format("%H:%M %d/%m/%Y");
                        row_strs.push(newdate.to_string());
                    }
                    ColumnField::AddedDate => {
                        let naive = NaiveDateTime::from_timestamp(torrent.added_date.unwrap(), 0);
                        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
                        let newdate = datetime.format("%H:%M %d/%m/%Y");
                        row_strs.push(newdate.to_string());
                    }
                }
            }
            rows.push(Row::new(row_strs));
        }
        let mut header_rows = Vec::new();
        for field in &self.all_info_columns {
            if !field.show {
                continue;
            }
            header_rows.push(field.column.to_str());
        }

        (header_rows, rows)
    }

    pub fn toggle_show_column(&mut self) {
        self.all_info_columns[self.selected_column.unwrap()].show =
            !self.all_info_columns[self.selected_column.unwrap()].show;
    }

    pub async fn rename_torrent(&mut self) {
        let client = TransClient::new("http://localhost:9091/transmission/rpc");
        client
            .torrent_rename_path(
                vec![Id::Id(self.get_selected_torrent_id())],
                self.get_selected_torrent_name(),
                self.input.to_owned(),
            )
            .await
            .unwrap();
    }

    pub async fn add_torrent(&mut self) {
        let client = TransClient::new("http://localhost:9091/transmission/rpc");
        let add: TorrentAddArgs = TorrentAddArgs {
            filename: Some(
                self.torrent_files[self.selected_torrent_file.unwrap()]
                    .to_str()
                    .unwrap()
                    .to_owned(),
            ),
            paused: Some(self.add_paused),
            ..TorrentAddArgs::default()
        };
        client.torrent_add(add).await.unwrap();
    }

    pub async fn remove_torrent(&mut self) {
        let client = TransClient::new("http://localhost:9091/transmission/rpc");
        client
            .torrent_remove(
                vec![Id::Id(self.get_selected_torrent_id())],
                self.delete_files,
            )
            .await
            .unwrap();
    }

    pub fn get_torrent_files(&mut self) {
        self.torrent_files.clear();

        match fs::read_dir("/home/kiipuri/Downloads") {
            Err(e) => log::error!("{}", e),
            Ok(paths) => {
                for path in paths {
                    let path = path.unwrap().path();
                    let file = Path::new(&path);
                    if file.extension().and_then(OsStr::to_str) == Some("torrent") {
                        self.torrent_files.push(file.to_path_buf());
                    }
                }
            }
        }
    }

    pub fn toggle_add_torrent_paused(&mut self) {
        self.add_paused = !self.add_paused;
    }

    fn get_selected_torrent_id(&self) -> i64 {
        self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
            .id
            .unwrap()
    }

    pub fn get_selected_torrent_name(&self) -> String {
        self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
            .name
            .to_owned()
            .unwrap()
    }

    pub fn show_tree(&mut self) {
        self.torrent_collapse_files.clear();
        let mut files = Vec::new();
        let mut dir = self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
            .download_dir
            .as_ref()
            .unwrap()
            .to_owned();

        dir.push_str(&self.get_selected_torrent_name());
        // TODO: loop torrent.files

        let walker = WalkDir::new(dir)
            .sort_by(|a, b| b.path().is_dir().cmp(&a.path().is_dir()))
            .into_iter();

        for entry in walker {
            let path = CollapsePath {
                path: entry.unwrap(),
                collapse: false,
            };
            files.push(path);
        }

        let mut collapse = false;
        let mut depth = 1;

        for entry in files {
            if entry.collapse && !collapse {
                collapse = true;
                depth = entry.path.depth();
            } else if collapse {
                if entry.path.depth() == depth {
                    collapse = false;
                }
            }

            if !collapse {
                // log::error!("{}", entry.path.path().display());
                self.torrent_collapse_files.push(entry);
            }
        }
    }
}

pub async fn get_all_torrents(app: &Arc<Mutex<App>>) {
    let client = TransClient::new("http://localhost:9091/transmission/rpc");
    let mut torrents = client.torrent_get(None, None).await.unwrap();
    torrents.arguments.torrents.sort_by(|a, b| {
        a.name
            .as_ref()
            .unwrap()
            .to_lowercase()
            .cmp(&b.name.as_ref().unwrap().to_lowercase())
    });

    let mut app = app.lock().unwrap();

    if !app.sort_descending {
        torrents.arguments.torrents.reverse();
    }
    app.torrents = torrents;
}
