#[derive(Debug, Default)]
pub struct Torrent {
    pub id: u32,
    pub name: String,
    pub hash: String,
    pub magnet: String,
    pub labels: String,
    pub state: String,
    pub location: String,
    pub percent_done: String,
    pub eta: String,
    pub download_speed: String,
    pub upload_speed: String,
    pub have: String,
    pub availability: String,
    pub total_size: String,
    pub downloaded: String,
    pub uploaded: String,
    pub ratio: String,
    pub corrupt_dl: String,
    pub peers: String,
    pub date_added: String,
    pub date_started: String,
    pub latest_activity: String,
    pub downloading_time: String,
    pub seeding_time: String,
    pub public_torrent: String,
    pub piece_count: String,
    pub piece_size: String,
    pub download_limit: String,
    pub upload_limit: String,
    pub ratio_limit: String,
    pub honors_session_limits: String,
    pub peer_limit: String,
    pub bandwidth_priority: String,
}

#[derive(Default)]
pub struct TorrentFile {
    pub id: u32,
    pub done: String,
    pub priority: String,
    pub get: String,
    pub size: String,
    pub name: String,
}

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
    pub torrents: Vec<Torrent>,
    pub selected_torrent: Option<usize>,
    pub floating_widget: FloatingWidget,
    pub torrent_files: Vec<TorrentFile>,
}

impl Default for App {
    fn default() -> Self {
        App {
            navigation_stack: vec![Route {
                id: RouteId::TorrentList,
                focused_widget: FocusableWidget::TorrentList,
            }],
            torrents: vec![],
            selected_torrent: Some(0),
            floating_widget: FloatingWidget::None,
            torrent_files: vec![],
        }
    }
}

impl App {
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
                if i >= self.torrents.len() - 1 {
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
                    Some(self.torrents.len() - 1)
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

    pub fn get_torrent_files(&mut self, id: u32) {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("transmission-remote -t {} --info-files", 2))
            .output();

        if let Ok(i) = output {
            let output = String::from_utf8_lossy(&i.stdout);

            let mut splits = output.split("\n").collect::<Vec<&str>>();
            splits.pop();
            for line in &splits[2..] {
                let mut file = TorrentFile::default();
                let mut line = line.trim().to_owned();
                while line != line.replace("  ", " ") {
                    line = line.replace("  ", " ");
                }

                let splits = line.split(" ").collect::<Vec<&str>>();
                file.id = splits[0][..splits[0].len() - 1].parse::<u32>().unwrap();
                file.done = splits[1].to_string();
                file.priority = splits[2].to_string();
                file.get = splits[3].to_string();
                file.size = format!("{} {}", splits[4].to_string(), splits[5].to_string());
                file.name = splits[6..].join(" ");

                self.torrent_files.push(file);
            }
        }
    }

    pub fn get_all_torrents(&mut self) {
        self.torrents = vec![];
        let mut counter = 1;
        loop {
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("transmission-remote -t {} -i", counter))
                .output();

            if let Ok(i) = output {
                let output = String::from_utf8_lossy(&i.stdout);
                if output.len() == 0 {
                    break;
                }

                let mut torrent = Torrent::default();

                for mut line in output.split("\n") {
                    if line.len() == 0 || !line.contains(":") {
                        continue;
                    }

                    line = &line[2..];
                    let splits = line.split(": ").collect::<Vec<&str>>();
                    let key = splits[0];
                    let value = splits[1].to_string();

                    match key {
                        "Id" => {
                            if let Ok(i) = value.parse::<u32>() {
                                torrent.id = i
                            }
                        }
                        "Name" => torrent.name = value,
                        "Hash" => torrent.hash = value,
                        "Magnet" => torrent.magnet = value,
                        "Labels" => torrent.labels = value,
                        "State" => torrent.state = value,
                        "Location" => torrent.location = value,
                        "Percent Done" => torrent.percent_done = value,
                        "ETA" => torrent.eta = value,
                        "Download Speed" => torrent.download_speed = value,
                        "Upload Speed" => torrent.upload_speed = value,
                        "Have" => torrent.have = value,
                        "Availability" => torrent.availability = value,
                        "Total size" => torrent.total_size = value,
                        "Downloaded" => torrent.downloaded = value,
                        "Uploaded" => torrent.uploaded = value,
                        "Ratio" => torrent.ratio = value,
                        "Corrupt DL" => torrent.corrupt_dl = value,
                        "Peers" => torrent.peers = value,
                        "Date added" => torrent.date_added = value,
                        "Date started" => torrent.date_started = value,
                        "Latest activity" => torrent.latest_activity = value,
                        "Downloading Time" => torrent.downloading_time = value,
                        "Seeding Time" => torrent.seeding_time = value,
                        "Public torrent" => torrent.public_torrent = value,
                        "Piece Count" => torrent.piece_count = value,
                        "Piece Size" => torrent.piece_size = value,
                        "Download Limit" => torrent.download_limit = value,
                        "Upload Limit" => torrent.upload_limit = value,
                        "Ratio Limit" => torrent.ratio_limit = value,
                        "Honors Session Limits" => torrent.honors_session_limits = value,
                        "Peer limit" => torrent.peer_limit = value,
                        "Bandwidth Priority" => torrent.bandwidth_priority = value,
                        _ => (),
                    }
                }
                self.torrents.push(torrent);
            }
            counter += 1;
        }
    }
}
