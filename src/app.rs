#[derive(Debug)]
pub struct Torrent {
    pub id: u32,
    pub name: String,
}

pub enum Route {
    Overview,
    TorrentInfo,
}

pub enum FocusableWidget {
    TorrentList,
    Help,
    None,
}

pub struct App {
    pub navigation_stack: Vec<Route>,
    pub torrents: Vec<Torrent>,
    pub selected_torrent: Option<usize>,
    pub focused_widget: FocusableWidget,
}

impl Default for App {
    fn default() -> Self {
        App {
            navigation_stack: vec![Route::Overview],
            torrents: vec![],
            selected_torrent: Some(0),
            focused_widget: FocusableWidget::TorrentList,
        }
    }
}

impl App {
    fn next_previous_match(&mut self, i: Option<usize>) {
        match self.focused_widget {
            FocusableWidget::TorrentList => self.selected_torrent = i,
            _ => (),
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
        self.set_focused_widget_on_stack_change();
    }

    pub fn stack_pop(&mut self) {
        self.navigation_stack.pop();
        self.set_focused_widget_on_stack_change();
    }

    fn set_focused_widget_on_stack_change(&mut self) {
        let widget = match self.navigation_stack.last() {
            Some(Route::Overview) => FocusableWidget::TorrentList,
            Some(Route::TorrentInfo) => FocusableWidget::None,
            _ => FocusableWidget::None,
        };

        self.focused_widget = widget;
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

                let mut torrent = Torrent {
                    id: 0,
                    name: "you should not see this".to_string(),
                };

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
                        _ => (),
                    }
                }
                self.torrents.push(torrent);
            }
            counter += 1;
        }
    }
}
