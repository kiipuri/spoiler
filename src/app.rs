use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use transmission_rpc::{
    types::{
        Id, RpcResponse, SessionGet, Torrent, TorrentAction, TorrentAddArgs, TorrentGetField,
        Torrents,
    },
    TransClient,
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
    None,
}

pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
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
    }

    pub fn previous(&mut self) {
        if self.selected_torrent > Some(0) {
            self.selected_torrent = Some(self.selected_torrent.unwrap() - 1);
        } else {
            self.selected_torrent = Some(self.torrents.arguments.torrents.len() - 1);
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
            (self.selected_file.unwrap() + 1)
                % self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
                    .files
                    .as_ref()
                    .unwrap()
                    .len(),
        );
    }

    pub fn previous_file(&mut self) {
        if self.selected_file.unwrap() > 0 {
            self.selected_file = Some(self.selected_file.unwrap() - 1);
        } else {
            self.selected_file = Some(
                self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
                    .files
                    .as_ref()
                    .unwrap()
                    .len()
                    - 1,
            )
        }
    }

    pub async fn increment_priority(&mut self) {
        self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
            .priorities
            .as_mut()
            .unwrap()[self.selected_file.unwrap()] = 1i8;
    }

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

    pub async fn add_torrent(&mut self, paused: bool) {
        let client = TransClient::new("http://localhost:9091/transmission/rpc");
        let add: TorrentAddArgs = TorrentAddArgs {
            filename: Some(
                self.torrent_files[self.selected_torrent_file.unwrap()]
                    .to_str()
                    .unwrap()
                    .to_owned(),
            ),
            paused: Some(paused),
            ..TorrentAddArgs::default()
        };
        client.torrent_add(add).await.unwrap();
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

    fn get_selected_torrent_id(&mut self) -> i64 {
        self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
            .id
            .unwrap()
    }

    fn get_selected_torrent_name(&mut self) -> String {
        self.torrents.arguments.torrents[self.selected_torrent.unwrap()]
            .name
            .to_owned()
            .unwrap()
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
