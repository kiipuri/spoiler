use transmission_rpc::{
    types::{RpcResponse, Torrent, Torrents},
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
    None,
}

pub enum FloatingWidget {
    Help,
    None,
}

pub struct App {
    pub navigation_stack: Vec<Route>,
    pub torrents: RpcResponse<Torrents<Torrent>>,
    pub selected_torrent: Option<usize>,
    pub floating_widget: FloatingWidget,
    pub should_quit: bool,
    pub torrent_client: TransClient,
}

impl App {
    pub fn new(torrent_client: TransClient, torrents: RpcResponse<Torrents<Torrent>>) -> Self {
        Self {
            navigation_stack: vec![Route {
                id: RouteId::TorrentList,
                focused_widget: FocusableWidget::TorrentList,
            }],
            floating_widget: FloatingWidget::None,
            selected_torrent: Some(0),
            should_quit: false,
            torrent_client,
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

    // pub async fn get_all_torrents_loop(&mut self) {
    //     tokio::spawn(async move {
    //         self.torrents = vec![];
    //         loop {
    //             tokio::time::sleep(Duration::from_secs(1)).await;
    //             let loop_res = self.torrent_client.torrent_get(None, None).await;
    //
    //             let mut torrents = res.lock().unwrap();
    //             if let Ok(i) = loop_res {
    //                 *torrents = i;
    //             }
    //             drop(torrents);
    //         }
    //     });
    // }
}
