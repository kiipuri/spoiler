use std::sync::{Arc, Mutex};

use transmission_rpc::{
    types::{RpcResponse, SessionGet, Torrent, Torrents},
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

    fn next_previous_match(&mut self, i: Option<usize>) {
        if let Some(a) = self.navigation_stack.last() {
            match a.focused_widget {
                FocusableWidget::TorrentList => self.selected_torrent = i,
                _ => (),
            }
        }
    }

    pub fn next(&mut self) {
        let i = match self.selected_torrent {
            Some(i) => {
                if i >= self.torrents.arguments.torrents.len() - 1 {
                    Some(0)
                } else {
                    Some(i + 1)
                }
            }
            None => Some(0),
        };

        self.next_previous_match(i);
    }

    pub fn previous(&mut self) {
        let i = match self.selected_torrent {
            Some(i) => {
                if i == 0 {
                    Some(self.torrents.arguments.torrents.len() - 1)
                } else {
                    Some(i - 1)
                }
            }
            None => Some(0),
        };
        self.next_previous_match(i);
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
