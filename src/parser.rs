#[derive(Clone, Debug)]
pub enum Command {
    ScrollUp,
    ScrollDown,
    JumpToStart,
    JumpToEnd,
    ZoomIn,
    ZoomOut,
}

#[derive(Clone, Debug, Default)]
struct State {
    value: String,
    command: Option<Command>,
    next: Vec<State>,
}

impl State {
    fn find(&self, word: &String) -> Option<State> {
        for s in &self.next {
            if s.value.eq(word) {
                return Some(s.clone());
            }
        }
        None
    }
}

#[derive(Default)]
pub struct KeyParser {
    state: State,
    root: State,
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
            command: Option::from(Command::ScrollUp),
            ..Default::default()
        };

        let k_state = State {
            value: String::from("k"),
            command: Option::from(Command::ScrollDown),
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

        let big_g_state = State{
            value: String::from("G"),
            command: Option::from(Command::JumpToEnd),
            ..Default::default()
        };

        // This is intended to be the last defined node.
        // New nodes should be defined above.
        self.root = State::default();

        // Please define edges below by adding nodes to the respective next Vector.
        g_state.next.push(gg_state);

        self.root.next.push(g_state);
        self.root.next.push(j_state);
        self.root.next.push(k_state);
        self.root.next.push(plus_state);
        self.root.next.push(minus_state);
        self.root.next.push(big_g_state);

        self.state = self.root.clone();
    }

    pub fn read(&mut self, input: String) -> Option<Command> {
        let word: String = self.state.value.clone() + &*input;
        let next_state = self.state.find(&word);
        let mut output: Option<Command> = None;
        match next_state {
            Some(state) => {
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
        output
    }
}
