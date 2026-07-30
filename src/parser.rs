#[derive(Clone, Debug)]
pub enum Command {
    ScrollUp,
    ScrollDown,
    JumpToStart,
    JumpToEnd,
    ZoomIn,
    ZoomOut,
    JumpToPage(usize),
}

#[derive(Clone, Debug, Default)]
struct State {
    value: String,
    command: Option<Command>,
    next: Vec<State>,
    number_sensitive: bool,
}

impl State {
    fn find(&self, word: &String, number_sensitive: bool) -> Option<State> {
        for s in &self.next {
            if number_sensitive {
                if s.value.eq(word) && s.number_sensitive {
                    return Option::from(s.clone());
                }
            } else {
                if s.value.eq(word) && !s.number_sensitive {
                    return Option::from(s.clone());
                }
            }
        }
        None
    }
}

#[derive(Default)]
pub struct KeyParser {
    state: State,
    root: State,
    num_buffer: usize,
}

impl KeyParser {
    pub fn init(&mut self) {
        let gg_state = State {
            value: String::from("gg"),
            command: Option::from(Command::JumpToStart),
            ..Default::default()
        };

        let mut g_state = State {
            value: String::from("g"),
            command: None,
            ..Default::default()
        };

        let j_state = State {
            value: String::from("j"),
            command: Option::from(Command::ScrollDown),
            ..Default::default()
        };

        let k_state = State {
            value: String::from("k"),
            command: Option::from(Command::ScrollUp),
            ..Default::default()
        };

        let plus_state = State {
            value: String::from("+"),
            command: Option::from(Command::ZoomIn),
            ..Default::default()
        };

        let minus_state = State {
            value: String::from("-"),
            command: Option::from(Command::ZoomOut),
            ..Default::default()
        };

        let big_g_state = State {
            value: String::from("G"),
            command: Option::from(Command::JumpToEnd),
            ..Default::default()
        };

        let gg_to_page_state = State {
            value: String::from("gg"),
            command: Option::from(Command::JumpToPage(0)),
            number_sensitive: true,
            ..Default::default()
        };

        let mut g_to_page_state = State {
            value: String::from("g"),
            command: None,
            number_sensitive: true,
            ..Default::default()
        };

        // This is intended to be the last defined node.
        // New nodes should be defined above.
        self.root = State::default();

        // Please define edges below by adding nodes to the respective next Vector.
        g_state.next.push(gg_state);

        g_to_page_state.next.push(gg_to_page_state);

        self.root.next.push(g_state);
        self.root.next.push(j_state);
        self.root.next.push(k_state);
        self.root.next.push(plus_state);
        self.root.next.push(minus_state);
        self.root.next.push(big_g_state);
        self.root.next.push(g_to_page_state);

        self.state = self.root.clone();
    }

    pub fn read(&mut self, letter: String) -> Option<Command> {
        if letter.parse::<usize>().is_ok() {
            let num = self.num_buffer.clone().to_string() + &*letter.to_string();

            // Try parsing num_buffer.
            // If too big for usize, start again from the current entry (so from letter).
            if let Ok(value) = num.parse::<usize>() {
                self.num_buffer = value
            } else {
                if let Ok(value) = letter.parse::<usize>() {
                    self.num_buffer = value;
                }
            }
            None
        } else {
            let mut output: Option<Command> = None;

            let word: String = self.state.value.clone() + &*letter;
            if self.num_buffer != 0 {
                let next_state = self.state.find(&word, true);

                // Did we reach a new state ?
                match next_state {
                    Some(state) =>
                    // Does our new state have a command ?
                    {
                        match state.command {
                            None => {
                                self.state = state.clone();
                            }
                            Some(Command::JumpToPage(0)) => {
                                output = Option::from(Command::JumpToPage(self.num_buffer - 1));
                                self.num_buffer = 0;
                                self.state = self.root.clone();
                            }
                            // number-sensitive commands must be
                            // handled explicitly.
                            _ => {
                                panic!(
                                    "number-sensitive command is \
                                                currently unhandled: {:?}",
                                    state.command
                                );
                            }
                        }
                    }
                    None => {
                        self.num_buffer = 0;
                        // We can discard the number as there is no number-sensitive
                        // state reachable with this word.
                        self.read(letter);
                    }
                }
            } else {
                let next_state = self.state.find(&word, false);

                // Did we reach a new state ?
                match next_state {
                    Some(state) =>
                    // Does our new state have a command ?
                    {
                        match state.command {
                            None => {
                                self.state = state.clone();
                            }
                            _ => {
                                output = Option::from(state.command.clone());
                                self.state = self.root.clone();
                            }
                        }
                    }
                    None => self.state = self.root.clone(),
                }
            }
            output
        }
    }
}
