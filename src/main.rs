use tcod::colors::*;
use tcod::console::*;
use tcod::input::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const MAP_WIDTH: usize = 80;
const MAP_HEIGHT: usize = 50;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};

const LIMIT_FPS: i32 = 20;

struct Tcod {
    root: Root,
    con: Offscreen,
}
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

    let player = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', WHITE);
    let npc = Object::new(SCREEN_WIDTH / 2 - 10, SCREEN_HEIGHT / 2, '@', YELLOW);

    let mut objects = [player, npc];

    let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);

    map.data[1] = Tile::wall();
    map.data[2] = Tile::wall();
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
// type Map = Vec<Tile>;

#[derive(Debug)]
struct Map {
    data: Vec<Tile>,
    w: usize,
    h: usize,
}

impl Map {
    pub fn new(w: usize, h: usize) -> Self {
        Map {
            data: vec![Tile::empty(); w * h],
            w,
            h,
        }
    }

    pub fn get_idx_pos(&self, i: usize) -> (i32, i32) {
        let x = i % self.w;
        let y = i / self.w;

        (x as i32, y as i32)
    }

    pub fn within(&self, x: i32, y: i32) -> bool {
        return x >= 0 && x < (self.w as i32) && y >= 0 && y < (self.h as i32);
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
        if game.map.within(_x, _y)
            && !game.map.data[_x as usize + game.map.w * _y as usize].blocked
        {
            self.x = _x;
            self.y = _y;
        }
    }

    pub fn draw(&self, con: &mut Console) {
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
