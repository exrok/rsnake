extern crate ncurses;
extern crate rand;

use rand::Rng;

use std::char;
use ncurses::*;
use std::collections::VecDeque;





#[derive(Debug)]
#[derive(Clone)]
enum Direction {
    Left,
    Down,
    Up,
    Right,
}

#[derive(Debug)]
#[derive(Clone)]
enum Symbol {
    Empty,
    BodyV,
    BodyH,
    BodyUL,
    BodyLL,
    BodyUR,
    BodyLR,
    Food,
    Head,
    HeadR,
    Border,
}

impl Symbol {
    fn value(&self) -> chtype {
        use Symbol::*;
        match *self {
            Empty => ' ' as chtype,
            BodyV => ACS_VLINE(),
            BodyH => ACS_HLINE(),
            BodyUL => ACS_ULCORNER(),
            BodyLL => ACS_LLCORNER(),
            BodyUR => ACS_URCORNER(),
            BodyLR => ACS_LRCORNER(),
            Food => '*' as chtype,
            Head | HeadR => 'o' as chtype,
            Border => ACS_CKBOARD(),
        }
    }
}



#[derive(Clone)]

struct Grid {
    width: i32,
    height: i32,
    display: Vec<Vec<(Symbol, i16)>>,
    double_row: bool,
}

impl Grid {
    fn new(width: i32, height: i32) -> Grid {
        Grid {
            width: width,
            height: height,
            display: vec![vec![(Symbol::Empty,0); width as usize]; height as usize],
            double_row: true,
        }
    }
    fn update(&mut self, pos: (i32, i32), sym: Symbol, colour: i16) {
        let (x, y) = pos;
        self.display[y as usize][x as usize] = (sym, colour);
    }

    fn update_nodes(&mut self, nodes: &Vec<((i32, i32), Symbol, i16)>) {
        for node in nodes {
            self.update(node.0, node.1.clone(), node.2);
        }
    }
    fn symbol(&self, pos: (i32, i32)) -> Symbol {
        let (x, y) = pos;
        self.display[y as usize][x as usize].0.clone()
    }
    fn add_border(&mut self) {
        let mut nodes = vec![];
        for x in 0..self.width {
            nodes.push(((x, 0), Symbol::Border, 2));
            nodes.push(((x, self.height - 1), Symbol::Border, 2));
        }
        for x in 0..self.height {
            nodes.push(((self.width - 1, x), Symbol::Border, 2));
            nodes.push(((0, x), Symbol::Border, 2));
        }
        self.update_nodes(&nodes);
    }

    fn drawgrid(&self) {
        use Symbol::*;
        for row in &self.display {
            for sym in *&row {
                attron(COLOR_PAIR(sym.1));
                addch(sym.0.value());
                if self.double_row {
                    addch(match sym.0 {
                        BodyH | BodyUL | BodyLL | HeadR => BodyH.value(),
                        Border => Border.value(),
                        _ => Empty.value(),
                    });
                }
                attroff(COLOR_PAIR(sym.1));
            }
            printw("\n");
        }
    }
}

struct Snake {
    body: VecDeque<(i32, i32)>,
    direction: Direction,
    length: u32,
    dirty_nodes: Vec<((i32, i32), Symbol, i16)>,
}

impl Snake {
    fn new(width: i32, height: i32, length: u32) -> Snake {
        let mut body = VecDeque::new();
        let center = (width / 2, height / 2);
        for _ in 0..length {
            body.push_back(center.clone());
        }
        Snake {
            body: body,
            direction: Direction::Up,
            length: length,
            dirty_nodes: Vec::new(),
        }
    }

    fn cut_tail(&mut self) {
        let tail_old = match self.body.pop_back() {
            Some(x) => x,
            _ => panic!("err"),
        };
        self.dirty_nodes.push((tail_old.clone(), Symbol::Empty, 0));

    }

    fn slither(&mut self, direction: Direction) -> (i32, i32) {
        use Direction::*;
        self.direction = direction;
        let prev_node = self.body[1];
        let head_old = self.body[0];


        let head_new = match self.direction {
            Left => (head_old.0 + 1, head_old.1),
            Down => (head_old.0, head_old.1 + 1),
            Up => (head_old.0, head_old.1 - 1),
            Right => (head_old.0 - 1, head_old.1),
        };
        self.body.push_front(head_new.clone());

        let last_dir = match (head_old.0 - prev_node.0, head_old.1 - prev_node.1) {
            (1, 0) => Right,
            (-1, 0) => Left,
            (0, 1) => Up,
            (0, -1) => Down,
            _ => self.direction.clone(),
        };
        let sym = match if (self.direction.clone() as i32) < (last_dir.clone() as i32) {
            (self.direction.clone(), last_dir.clone())
        } else {
            (last_dir.clone(), self.direction.clone())
        } {
            (Left, Right) => Symbol::BodyH,
            (Up, Right) => Symbol::BodyLR,
            (Left, Up) => Symbol::BodyLL,
            (Down, Up) => Symbol::BodyV,
            (Left, Down) => Symbol::BodyUL,
            (Down, Right) => Symbol::BodyUR,
            _ => Symbol::BodyV,
        };
        self.dirty_nodes.push((head_old.clone(), sym, 1));
        self.dirty_nodes.push((head_new.clone(),
                               match self.direction {
                                   Right => Symbol::HeadR,
                                   _ => Symbol::Head,

                               },
                               1));

        head_new
    }
}

struct Game {
    grid: Grid,
    speed: u32,
    snake: Snake,
    food: Option<(i32, i32)>,
}

impl Game {
    fn new() -> Game {
        Game {
            grid: Grid::new(40, 30),
            speed: 2,
            snake: Snake::new(40, 30, 3),
            food: None,
        }
    }

    fn init_ncurses(&self) {
        initscr();
        keypad(stdscr(), true);
        noecho();

        start_color();

        use_default_colors();
        assume_default_colors(-1, -1);

        init_pair(1, COLOR_RED, -1);
        init_pair(2, COLOR_WHITE, -1);
        init_pair(0, -1, -1);

        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

        halfdelay(self.speed as i32);

        refresh();
    }

    fn kill_ncurses(&self) {
        endwin();
    }

    fn place_food(&mut self) {
        self.food = Some((rand::thread_rng().gen_range(1, self.grid.width - 1),
                          rand::thread_rng().gen_range(1, self.grid.height - 1)));
        self.grid.update(self.food.unwrap(), Symbol::Food, 1);
    }

    fn get_input(&self) -> Option<Direction> {
        let ch = wget_wch(stdscr());
        match ch {
            Some(WchResult::KeyCode(code)) => {
                Some(match code {
                    260 => Direction::Right,
                    258 => Direction::Down,
                    259 => Direction::Up,
                    261 => Direction::Left,
                    _ => self.snake.direction.clone(),
                })
            }
            Some(WchResult::Char(c)) => {
                let pressed = char::from_u32(c as u32).expect("Invalid char");
                Some(match pressed {
                    'h' => Direction::Right,
                    'j' => Direction::Down,
                    'k' => Direction::Up,
                    'l' => Direction::Left,
                    _ => self.snake.direction.clone(),
                })
            }
            _ => None,
        }

    }

    fn draw(&mut self) {
        clear();
        // printw(format!("{:?}", self.snake.direction).as_ref());
        self.grid.drawgrid();
    }
    fn game_over(&self) {
        let outtexts = vec![format!("Game Over"),
                            format!("Score: {}", 123)];
        for (i, text) in outtexts.iter().enumerate() {
            mvprintw((self.grid.height / 2 + i as i32),
                     self.grid.width / 2,
                     format!("{text:^width$}\n",
                             text = &text,
                             width = self.grid.width as usize)
                         .as_ref());
        }
    }
    fn start(&mut self) {
        self.init_ncurses();
        self.place_food();
        self.grid.add_border();
        loop {

            let dir = match self.get_input() {
                Some(dir) => {

                    use Direction::*;
                    let odir = match self.snake.direction.clone(){
                        Right => Left,
                        Left => Right,
                        Up => Down,
                        Down => Up
                     };
                    if dir.clone() as u32 == odir as u32 {
                        self.snake.direction.clone()
                    }else{
                        dir
                    }
                },
                None => self.snake.direction.clone(), //Implement Wait Hear
            };

            let head_pos = self.snake.slither(dir);

            match self.grid.symbol(head_pos) {
                Symbol::Food => {
                    self.snake.length += 1;
                    self.place_food();
                }
                Symbol::Empty => self.snake.cut_tail(),
                _ => break,
            };

            self.grid.update_nodes(&self.snake.dirty_nodes);
            self.snake.dirty_nodes.clear();
            self.draw();
            refresh();

        }
        clear();
        self.game_over();
        nocbreak();
        refresh();
        getch();
        self.kill_ncurses();
    }
}

fn main() {
    let mut game = Game::new();
    game.start();


}
