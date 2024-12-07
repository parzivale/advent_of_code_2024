use std::{
    fs::File,
    io::{BufReader, Read},
    ops::ControlFlow,
};

use colored::{ColoredString, Colorize};

const GUARD_CHARS: [char; 4] = ['^', 'v', '<', '>'];

#[derive(Debug, Default, Clone, PartialEq, Eq)]
enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

impl From<char> for Direction {
    fn from(value: char) -> Self {
        match value {
            '^' => Direction::Up,
            'v' => Direction::Down,
            '<' => Direction::Left,
            '>' => Direction::Right,
            _ => panic!("Invalid direction"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct Guard {
    x: i32,
    y: i32,
    direction: Direction,
}

impl Guard {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            direction: Direction::Up,
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32, direction: impl Into<Direction>) {
        self.x = x;
        self.y = y;
        self.direction = direction.into();
    }

    pub fn move_forward(&mut self) {
        match self.direction {
            Direction::Up => self.y -= 1,
            Direction::Down => self.y += 1,
            Direction::Left => self.x -= 1,
            Direction::Right => self.x += 1,
        }
    }

    pub fn get_position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn rotate(&mut self) {
        self.direction = match self.direction {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }

    pub fn in_front(&self) -> (i32, i32) {
        match self.direction {
            Direction::Up => (self.x, self.y - 1),
            Direction::Down => (self.x, self.y + 1),
            Direction::Left => (self.x - 1, self.y),
            Direction::Right => (self.x + 1, self.y),
        }
    }
}

#[derive(Default, Debug)]
struct Dimensions {
    x: usize,
    y: usize,
}

#[derive(Default, Debug)]
struct Ground {
    layout: Vec<Vec<char>>,
    dimensions: Dimensions,
    obstacle: Option<(i32, i32)>,
    guard: Guard,
    unique: u32,
    temp: char,
}

impl Ground {
    pub fn get_character_at(&self, x: usize, y: usize) -> &char {
        self.layout
            .get(y)
            .unwrap_or_else(|| panic!("Not a valid y coord: {}", y))
            .get(x)
            .unwrap_or_else(|| panic!("Not a valid x coord: {}", x))
    }

    pub fn mutate_character_at(&mut self, x: usize, y: usize, c: char) {
        *self
            .layout
            .get_mut(y)
            .unwrap_or_else(|| panic!("Not a valid y coord: {}", y))
            .get_mut(x)
            .unwrap_or_else(|| panic!("Not a valid x coord: {}", x)) = c;
    }



    fn move_guard(&mut self) -> ControlFlow<(), Guard> {
        //println!("Obstacle before move_guard: {:?}", self.obstacle); // Add this line
        let (current_x, current_y) = self.guard.get_position();
        let (x, y) = self.guard.in_front();
        let x_dimensions = self.dimensions.x as i32;
        let y_dimensions = self.dimensions.y as i32;

        if x <= 0 || y <= 0 || x >= x_dimensions || y >= y_dimensions {
            ControlFlow::Break(())
        } else {
            let c = self.get_character_at(x as usize, y as usize).to_owned();
            if let Some((obs_x, obs_y)) = self.obstacle {
                if obs_x == x && obs_y == y {

                    self.guard.rotate();
                    return self.move_guard()
                }
            }

            match c {
                '.' => {
                    self.mutate_character_at(current_x as usize, current_y as usize, 'X');
                    self.unique += 1;
                }
                'X' => self.mutate_character_at(current_x as usize, current_y as usize, '0'),
                '#' => {
                    self.guard.rotate();
                    return self.move_guard()
                }
                _ => (),
            };

            self.guard.move_forward();

            ControlFlow::Continue(self.guard.clone())
        }
    }

    pub fn run_unqiue(&mut self) -> u32 {
        let mut history = vec![];
        loop {
            match self.move_guard() {
                ControlFlow::Break(_) => {
                    println!(
                        "{}",
                        self.layout
                            .iter()
                            .map(|x| x.iter().collect::<String>() + "\n")
                            .collect::<String>()
                            .replace("X", &"X".red())
                    );
                    break;
                }
                ControlFlow::Continue(guard) => {
                    if history.contains(&guard) {
                        println!(
                            "{}",
                            self.layout
                                .iter()
                                .map(|x| x.iter().collect::<String>() + "\n")
                                .collect::<String>()
                                .replace("X", &"X".red())
                        );
                        break;
                    }
                    history.push(guard);
                }
            }
        }

        self.unique + 1
    }

    pub fn run(&mut self) -> ControlFlow<()> {
        let mut history = vec![];
        loop {
            match self.move_guard() {
                ControlFlow::Break(_) => {
                    break;
                }
                ControlFlow::Continue(guard) => {
                    if history.contains(&guard) {
                         println!(
                             "{}",
                             self.layout
                                 .iter()
                                 .map(|x| x
                                     .iter()
                                     .map(|x| {
                                         match x {
                                             'X' => x.to_string().red().to_string(),
                                             '0' => x.to_string().green().to_string(),
                                             '@' => x.to_string().bright_blue().to_string(),
                                             _ => x.to_string().normal().to_string(),
                                         }
                                     })
                                     .collect::<String>()
                                     + "\n")
                                 .collect::<String>()
                         );
                        return ControlFlow::Break(());
                    }
                    history.push(guard);
                }
            };
        }

        ControlFlow::Continue(())
    }

    pub fn increment_obstacle(&mut self) -> ControlFlow<()> {
        let (x, y) = self.obstacle.unwrap();
        self.layout[y as usize][x as usize] = self.temp;

        if x + 1 >= self.dimensions.x as i32 {
            if y + 1 >= self.dimensions.y as i32 {
                return ControlFlow::Break(());
            } else {
                self.obstacle = Some((0, y + 1));
                self.temp = self.get_character_at(0, (y + 1) as usize).to_owned();
                self.layout[(y + 1) as usize][0] = '@';
            }
        } else {
            self.obstacle = Some((x + 1, y));
            self.temp = self
                .get_character_at((x + 1) as usize, y as usize)
                .to_owned();
            self.layout[y as usize][(x + 1) as usize] = '@';
        }

        //if self.get_character_at(x as usize, y as usize) == &'#' {
        //    self.increment_obstacle();
        //}



        ControlFlow::Continue(())
    }
}

fn main() {
    let f = File::open("./data/day_6/input").expect("file not found");
    let mut reader = BufReader::new(f);

    let mut group = String::new();
    reader
        .read_to_string(&mut group)
        .expect("unable to read file");

    let mut guard = Guard::new(0, 0);
    let layout: Vec<Vec<char>> = group
        .split("\n")
        .enumerate()
        .map(|(y, string)| {
            string
                .chars()
                .enumerate()
                .inspect(|(x, c)| {
                    if GUARD_CHARS.contains(c) {
                        guard.set_position(*x as i32, y as i32, *c);
                    }
                })
                .map(|(_, c)| c)
                .collect::<Vec<char>>()
        })
        .collect();

    let layout_length = layout.len();
    let layout_width = layout.first().unwrap().len();


    let mut ground = Ground {
        layout: layout.clone(),
        dimensions: Dimensions {
            y: layout_length,
            x: layout_width,
        },
        guard: guard.clone(),
        unique: 0,
        obstacle: Some((0, 0)),
        temp: '.',
    };

    println!("{}", ground.run_unqiue());



    let mut ground = Ground {
        layout: layout.clone(),
        dimensions: Dimensions {
            y: layout_length,
            x: layout_width,
        },
        guard: guard.clone(),
        unique: 0,
        obstacle: Some((0, 0)),
        temp: '.',
    };



    let mut count = 0;
    loop {

        if let ControlFlow::Break(_) = ground.run() {
            count += 1;
        };

        ground.layout = layout.clone();

        if let ControlFlow::Break(_) = ground.increment_obstacle() {
            break;
        };

        ground.guard = guard.clone();

    }
    println!("{}", count);





}
