pub struct LayerConnector {
    chars: Vec<Vec<&'static str>>,
    from: Vec<usize>,
    pub to: Vec<usize>,
}

impl LayerConnector {
    const FORWARD: &str = "─";
    const TURN_UP: &str = "╯";
    const TURN_DOWN: &str = "╮";
    const UP: &str = "│";
    const SPACE: &str = " ";
    const TURN_FORWAR_FROM_DOWN: &str = "╰";
    const TURN_FORWAR_FROM_UP: &str = "╭";

    pub fn new(height: usize, from: Vec<usize>) -> Self {
        Self {
            chars: vec![Vec::new(); height],
            from,
            to: Vec::new(),
        }
    }

    fn resolve_connection(&mut self, con: (usize, usize), cols: usize, up: bool) {
        let mut row = con.0;
        let mut col = 0;
        let mut last_was_forward = true;
        let target = con.1;

        while row != target || col != cols {
            let (new_row, new_col, new_is_forward) =
                if row > target && self.chars[row - 1][col] == Self::SPACE {
                    (row - 1, col, false)
                } else if row < target && self.chars[row + 1][col] == Self::SPACE {
                    (row + 1, col, false)
                } else {
                    (row, col + 1, true)
                };

            let unit = match (last_was_forward, new_is_forward, up) {
                (true, true, _) => Self::FORWARD,
                (true, false, true) => Self::TURN_UP,
                (true, false, false) => Self::TURN_DOWN,
                (false, true, true) => Self::TURN_FORWAR_FROM_UP,
                (false, true, false) => Self::TURN_FORWAR_FROM_DOWN,
                (false, false, _) => Self::UP,
            };

            self.chars[row][col] = unit;
            row = new_row;
            col = new_col;
            last_was_forward = new_is_forward;
        }
    }

    pub fn resolve(&mut self) {
        let mut upper_connections = Vec::new();
        let mut lower_connections = Vec::new();

        for (from, to) in self.from.iter().zip(&self.to) {
            if from > to {
                upper_connections.push((*from, *to));
            } else {
                lower_connections.push((*from, *to));
            }
        }

        let cols = upper_connections.len().max(lower_connections.len());

        for row in &mut self.chars {
            *row = vec![Self::SPACE; cols]
        }

        for con in upper_connections {
            self.resolve_connection(con, cols, true);
        }

        for con in lower_connections.iter().rev() {
            self.resolve_connection(*con, cols, false);
        }
    }

    pub fn get_row(&self, idx: usize) -> String {
        self.chars[idx].join("")
    }
}
