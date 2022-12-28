use math::round;

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
};

use transmission_rpc::{
    types::{
        Id, SessionGet, SessionStats, Torrent, TorrentAction, TorrentAddArgs, TorrentGetField,
    },
    TransClient,
};
use tui::widgets::Row;
use tui_tree_widget::TreeState;

use crate::{
    config::Config,
    conversion::{
        compare_float, compare_int, compare_string, convert_rate, convert_secs, date,
        get_status_percentage, status_string,
    },
    tree::{make_tree, StatefulTree},
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

#[derive(Clone, Copy)]
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
    pub fn as_str(&self) -> String {
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

pub struct ColumnAndShow {
    pub column: ColumnField,
    pub show: bool,
}

pub struct Data {
    pub download: Vec<(f64, f64)>,
    pub upload: Vec<(f64, f64)>,
    pub height: f64,
}

impl Data {
    fn new() -> Data {
        let data = vec![(10.0, 0.0)];
        let data1 = vec![(10.0, 0.0)];
        Data {
            download: data,
            upload: data1,
            height: 10000.0,
        }
    }

    pub fn on_tick(&mut self, download: i64, upload: i64) {
        if self.download.first().unwrap().0 <= 0.0 {
            self.download.remove(0);
        }
        if self.upload.first().unwrap().0 <= 0.0 {
            self.upload.remove(0);
        }
        let mut max_download = 0.0;
        for point in &mut self.download {
            point.0 -= 0.1;
            if max_download < point.1 {
                max_download = round::ceil(point.1, -1);
            }
        }

        let mut max_upload = 0.0;
        for point in &mut self.upload {
            point.0 -= 0.1;
            if max_upload < point.1 {
                max_upload = round::ceil(point.1, -point.1.log10().ceil() as i8);
            }
        }

        if max_download > max_upload {
            self.height = max_download;
        } else {
            self.height = max_upload;
        }

        let last_x = self.download.last().unwrap().0;
        self.download.push((last_x + 0.1, download as f64));
        self.upload.push((last_x + 0.1, upload as f64));
    }
}

pub struct App<'a> {
    pub session_stats: Option<SessionStats>,
    pub session: Option<SessionGet>,
    pub config: Config,
    pub navigation_stack: Vec<Route>,
    pub torrents: Vec<Torrent>,
    pub selected_torrent: Option<usize>,
    pub selected_tab: usize,
    pub floating_widget: FloatingWidget,
    pub should_quit: bool,
    pub sort_descending: bool,
    pub sort_column: ColumnField,
    pub input_mode: InputMode,
    pub input: String,
    pub torrent_files: Vec<PathBuf>,
    pub selected_torrent_file: Option<usize>,
    pub add_paused: bool,
    pub delete_files: bool,
    pub all_info_columns: Vec<ColumnAndShow>,
    pub selected_column: Option<usize>,
    pub tree: StatefulTree<'a>,
    pub data: Data,
}

impl<'a> App<'a> {
    pub async fn new() -> App<'a> {
        Self {
            session_stats: None,
            session: None,
            config: Config::new(),
            navigation_stack: vec![Route {
                id: RouteId::TorrentList,
                focused_widget: FocusableWidget::TorrentList,
            }],
            floating_widget: FloatingWidget::None,
            selected_torrent: Some(0),
            selected_tab: 0,
            should_quit: false,
            torrents: Vec::new(),
            sort_descending: true,
            sort_column: ColumnField::Name,
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
            tree: StatefulTree::new(),
            data: Data::new(),
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

    pub fn tree_with_path(&mut self) {
        let torrent = self.get_selected_torrent();
        let path_str = torrent.download_dir.as_ref().unwrap().to_owned();
        let mut path = PathBuf::from_str(&path_str).unwrap();
        let torrent_name_path = PathBuf::from_str(torrent.name.as_ref().unwrap());
        path = path.join(torrent_name_path.unwrap());
        let mut skipped_dirs: Vec<PathBuf> = Vec::new();
        self.tree.items = make_tree(path, self, &mut skipped_dirs);
        self.tree.state = TreeState::default();
    }

    pub fn next(&mut self) {
        self.selected_torrent = Some((self.selected_torrent.unwrap() + 1) % self.torrents.len());
        self.tree_with_path();
    }

    pub fn previous(&mut self) {
        if self.selected_torrent > Some(0) {
            self.selected_torrent = Some(self.selected_torrent.unwrap() - 1);
        } else {
            self.selected_torrent = Some(self.torrents.len() - 1);
        }
        self.tree_with_path();
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
        if self.torrent_files.is_empty() {
            return;
        }

        self.selected_torrent_file =
            Some((self.selected_torrent_file.unwrap() + 1) % self.torrent_files.len());
    }

    pub fn previous_torrent_file(&mut self) {
        if self.torrent_files.is_empty() {
            return;
        }

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

    pub async fn toggle_torrent_pause(&mut self) {
        let mut client = TransClient::new("http://localhost:9091/transmission/rpc");

        let id = self.torrents[self.selected_torrent.unwrap()].id.unwrap();

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
                    self.torrents[self.selected_torrent.unwrap()].id.unwrap(),
                )],
            )
            .await
            .unwrap();
    }

    pub fn get_torrent_rows(&self) -> (Vec<String>, Vec<Row>) {
        let mut rows = Vec::new();
        for torrent in &self.torrents {
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
                        row_strs.push(convert_secs(torrent.eta.unwrap()));
                    }
                    ColumnField::Status => {
                        row_strs.push(status_string(torrent.status.as_ref().unwrap()).to_string());
                    }
                    ColumnField::Progress => {
                        row_strs.push(get_status_percentage(torrent));
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
                        row_strs.push(date(torrent.done_date.unwrap()));
                    }
                    ColumnField::AddedDate => {
                        row_strs.push(date(torrent.added_date.unwrap()));
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
            header_rows.push(field.column.as_str());
        }

        (header_rows, rows)
    }

    pub fn toggle_show_column(&mut self) {
        self.all_info_columns[self.selected_column.unwrap()].show =
            !self.all_info_columns[self.selected_column.unwrap()].show;
    }

    pub async fn rename_torrent(&mut self) {
        let mut client = TransClient::new("http://localhost:9091/transmission/rpc");
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
        let mut client = TransClient::new("http://localhost:9091/transmission/rpc");
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
        let mut client = TransClient::new("http://localhost:9091/transmission/rpc");
        client
            .torrent_remove(
                vec![Id::Id(self.get_selected_torrent_id())],
                self.delete_files,
            )
            .await
            .unwrap();
    }

    pub async fn verify_torrent(&mut self) {
        let mut client = TransClient::new("http://localhost:9091/transmission/rpc");
        client
            .torrent_action(
                TorrentAction::Verify,
                vec![Id::Id(self.get_selected_torrent_id())],
            )
            .await
            .unwrap();
    }

    pub fn get_torrent_files(&mut self) {
        self.torrent_files.clear();

        if let Some(dir) = &self.config.torrent_search_dir {
            if let Ok(paths) = fs::read_dir(dir) {
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
        self.torrents[self.selected_torrent.unwrap()].id.unwrap()
    }

    pub fn get_selected_torrent(&self) -> &Torrent {
        &self.torrents[self.selected_torrent.unwrap()]
    }

    pub fn get_selected_torrent_name(&self) -> String {
        self.torrents[self.selected_torrent.unwrap()]
            .name
            .to_owned()
            .unwrap()
    }
}

pub async fn get_all_torrents<'a>(app: &Arc<Mutex<App<'a>>>) {
    let mut client = TransClient::new("http://localhost:9091/transmission/rpc");
    let mut torrents = client
        .torrent_get(None, None)
        .await
        .unwrap()
        .arguments
        .torrents;
    let session_stats = client.session_stats().await.unwrap().arguments;
    let session = client.session_get().await.unwrap().arguments;

    let mut app = app.lock().unwrap();

    torrents.sort_by(|a, b| match app.sort_column {
        ColumnField::Id => compare_int(a.id.unwrap(), b.id.unwrap()),
        ColumnField::Name => compare_string(a.name.as_ref().unwrap(), b.name.as_ref().unwrap()),
        ColumnField::Status => compare_int(a.status.unwrap(), b.status.unwrap()),
        ColumnField::Progress => compare_float(a.percent_done.unwrap(), b.percent_done.unwrap()),
        ColumnField::Eta => compare_int(a.eta.unwrap(), b.eta.unwrap()),
        ColumnField::UploadRate => compare_int(a.rate_upload.unwrap(), b.rate_upload.unwrap()),
        ColumnField::DownloadRate => {
            compare_int(a.rate_download.unwrap(), b.rate_download.unwrap())
        }
        ColumnField::UploadRatio => compare_float(a.upload_ratio.unwrap(), b.upload_ratio.unwrap()),
        ColumnField::DoneDate => compare_int(a.done_date.unwrap(), b.done_date.unwrap()),
        ColumnField::AddedDate => compare_int(a.added_date.unwrap(), b.added_date.unwrap()),
    });

    if !app.sort_descending {
        torrents.reverse();
    }
    app.torrents = torrents;
    app.session_stats = Some(session_stats);
    app.session = Some(session);
}
