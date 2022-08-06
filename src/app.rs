use std::sync::{Arc, Mutex};

use transmission_rpc::{
    types::{RpcResponse, SessionGet, Torrent, TorrentAddArgs, Torrents},
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
    None,
}

pub struct App {
    pub navigation_stack: Vec<Route>,
    pub torrents: RpcResponse<Torrents<Torrent>>,
    pub selected_torrent: Option<usize>,
    pub selected_tab: usize,
    pub selected_file: Option<usize>,
    pub floating_widget: FloatingWidget,
    pub should_quit: bool,
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

        let client = TransClient::new("http://localhost:9091/transmission/rpc");
        let add = TorrentAddArgs {
            filename: Some("archlinux-2022.07.01-x86_64.iso.torrent".to_string()),
            ..TorrentAddArgs::default()
        };
        if let Err(_) = client.torrent_add(add).await {
            println!("fucked up");
        };

        // client
        //     .torrent_remove(vec![transmission_rpc::types::Id::Id(0)], false)
        //     .await;
    }
}

pub async fn get_all_torrents_loop(app: Arc<Mutex<App>>) {
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
    app.torrents = torrents;
}
