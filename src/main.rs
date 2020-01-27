use rand::Rng;
use std::cmp::*;
use tcod::colors::*;
use tcod::console::*;
use tcod::input::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 50;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};

//parameters for dungeon generator
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

const LIMIT_FPS: i32 = 20;

struct Tcod {
    root: Root,
    con: Offscreen,
}

// handle user input
fn handle_keys(tcod: &mut Tcod, o: &mut Object, game: &Game) -> bool {
    let key = tcod.root.wait_for_keypress(true);
    match key {
        // for controlling players
        Key {
            code: KeyCode::Left,
            ..
        } => o.move_by(-1, 0, game),
        Key {
            code: KeyCode::Right,
            ..
        } => o.move_by(1, 0, game),
        Key {
            code: KeyCode::Up, ..
        } => o.move_by(0, -1, game),
        Key {
            code: KeyCode::Down,
            ..
        } => o.move_by(0, 1, game),

        // for full screen settings
        Key {
            code: KeyCode::Enter,
            alt: true,
            ..
        } => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        Key {
            code: KeyCode::Escape,
            ..
        } => {
            return true;
        }
        _ => {}
    }
    false
}

// draw everything
fn draw(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    // draw map first
    for i in 0..(game.map.w * game.map.h) {
        let (x, y) = game.map.get_idx_pos(i);
        let tile = &game.map.data[i];
        if tile.block_sight {
            tcod.con
                .set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
        } else {
            tcod.con
                .set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
        }
    }
    // draw each object
    for o in objects {
        o.draw(&mut tcod.con);
    }
}

// map generate

fn make_map(player: &mut Object) -> Map {
    let mut map = Map::new(MAP_WIDTH as usize, MAP_HEIGHT as usize);
    let mut rooms = vec![];

    for _i in 0..MAX_ROOMS {
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - ROOM_MAX_SIZE);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - ROOM_MAX_SIZE);
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE);

        let new_room = Rect::new(x, y, w, h);
        let is_intersected = rooms
            .iter()
            .any(|other_room| new_room.is_intersected(other_room));

        if !is_intersected {
            let (cur_x, cur_y) = new_room.center();
            if rooms.is_empty() {
                // if it's the first room, place player in the center of room
                player.x = cur_x;
                player.y = cur_y;
            } else {
                // if not, connect to previous room
                let prev_room = &rooms[rooms.len() - 1];
                let (prev_x, prev_y) = prev_room.center();

                // create tunnel
                if rand::random() {
                    // vertical first
                    map.create_v_tunnel(prev_y, cur_y, prev_x);
                    map.create_h_tunnel(prev_x, cur_x, cur_y);
                } else {
                    // horizontal first
                    map.create_h_tunnel(prev_x, cur_x, prev_y);
                    map.create_v_tunnel(prev_y, cur_y, cur_x);
                }
            }

            // append to rooms
            map.create_room(new_room);
            rooms.push(new_room);
        }
    }

    map
}

fn main() {
    println!("Hello, RoughLike!");

    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("My RogueLike")
        .init();

    let con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    let mut tcod = Tcod {
        root: root,
        con: con,
    };

    // create the map
    let player = Object::new(5, 5, '@', WHITE);
    let mut objects = [player];

    let map = make_map(&mut objects[0]);

    // map.data[101] = Tile::wall();
    let game = Game { map };

    // player states
    tcod::system::set_fps(LIMIT_FPS);

    while !tcod.root.window_closed() {
        // draw each objects
        tcod.con.clear();

        draw(&mut tcod, &game, &objects);

        // blit & flush
        blit(
            &tcod.con,
            (0, 0),
            (MAP_WIDTH as i32, MAP_HEIGHT as i32),
            &mut tcod.root,
            (0, 0),
            1.0,
            1.0,
        );
        tcod.root.flush();

        // handle input and update
        let exit = handle_keys(&mut tcod, &mut objects[0], &game);
        if exit {
            break;
        }
    }
}

// Map
#[derive(Debug)]
struct Map {
    data: Vec<Tile>,
    w: usize,
    h: usize,
}

impl Map {
    pub fn new(w: usize, h: usize) -> Self {
        Map {
            data: vec![Tile::wall(); w * h],
            w,
            h,
        }
    }

    // given an index in data array
    // return position(x, y) in 2d cordinate
    pub fn get_idx_pos(&self, i: usize) -> (i32, i32) {
        let x = i % self.w;
        let y = i / self.w;

        (x as i32, y as i32)
    }

    // given a position(x, y) in 2d cordinate
    // return index in the map `data` array
    pub fn get_pos_idx(&self, x: i32, y: i32) -> usize {
        x as usize + y as usize * self.w
    }

    pub fn within(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < (self.w as i32) && y >= 0 && y < (self.h as i32)
    }

    // create a `Rectangle Room` within the map
    pub fn create_room(&mut self, rect: Rect) {
        for x in (rect.x1 + 1)..rect.x2 {
            for y in (rect.y1 + 1)..rect.y2 {
                let idx = self.get_pos_idx(x, y);
                self.data[idx] = Tile::empty();
            }
        }
    }

    // create a horizontal tunnel
    pub fn create_h_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..(max(x1, x2) + 1) {
            let idx = self.get_pos_idx(x, y);
            self.data[idx] = Tile::empty();
        }
    }

    // create a vertical tunnel
    pub fn create_v_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..(max(y1, y2) + 1) {
            let idx = self.get_pos_idx(x, y);
            self.data[idx] = Tile::empty();
        }
    }
}

// Game Struct
#[derive(Debug)]
struct Game {
    map: Map,
}

// Object
#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object { x, y, char, color }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        let _x = self.x + dx;
        let _y = self.y + dy;
        if game.map.within(_x, _y) && !game.map.data[_x as usize + game.map.w * _y as usize].blocked
        {
            self.x = _x;
            self.y = _y;
        }
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

// Tile
#[derive(Debug, Clone)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
        }
    }
}

// rect
#[derive(Debug, Copy, Clone)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x1: i32, y1: i32, w: i32, h: i32) -> Self {
        Rect {
            x1,
            y1,
            x2: x1 + w,
            y2: y1 + h,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    pub fn is_intersected(&self, other: &Rect) -> bool {
        !(other.x2 < self.x1 || other.x1 > self.x2 || other.y2 < self.y1 || other.y1 > self.y2)
    }
}
