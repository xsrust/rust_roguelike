use std::cmp;
use tcod::colors::*;
use tcod::console::*;

/*********  CONSTANTS  **********/
/// Actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
/// size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;
/// Other settings
const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum
/// Colors
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};

/*********  STRUCTURES  *********/
struct Tcod {
    root: Root,
    con: Offscreen,
}

/// This is a generic object: the player, a monster, an item, the stairs...
/// It's always represented by a charactor on the screen.
#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

/// A tile of the map and it's prooerties
#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

/// a list of list of tiles, to represent the map
type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}

/// A rectangle on the map, used to characterise a room.
#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

fn main() {
    tcod::system::set_fps(LIMIT_FPS);

    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();
    let con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    let mut tcod = Tcod { root, con };

    // Initilize Player Properties
    let player = Object::new(25, 23, '@', WHITE);
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', YELLOW);

    // List of objects in the game, currently player, npc
    let mut objects = [player, npc];
    let game = Game { map: make_map() };

    while !tcod.root.window_closed() {
        // handle the updating of the view port
        render_all(&mut tcod, &game, &objects);

        // Key handleing w/ exit
        let player = &mut objects[0];
        let exit = handle_keys(&mut tcod, player, &game);
        if exit {
            break;
        }
    }
}

/// Fucntions for the object structure
impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object { x, y, char, color }
    }

    /// move by the given amount
    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {
            self.x += dx;
            self.y += dy;
        }
    }

    /// set the color and then draw the character that represents this object at it's
    /// position on the screen
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

/// Functions for the tile structure
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

/// Functinos for the rectangle structure
impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            x2: x + w,
            y1: y,
            y2: y + h,
        }
    }
}

/// render all of the things
fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    tcod.con.clear(); // clean console
                      // Draw all of our objects on the screen
    for object in objects {
        object.draw(&mut tcod.con);
    }
    // Draw all our tiles onto the screen
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].block_sight;
            if wall {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
        }
    }
    tcod.root.flush(); // clean root console to write to
                       // blit the contents of "con" onto the root console and present it
    blit(
        &tcod.con,
        (0, 0),
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}

/// Create our Map object
fn make_map() -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    // create the rooms
    let room1 = Rect::new(20, 15, 10, 15);
    let room2 = Rect::new(50, 15, 10, 15);
    create_room(room1, &mut map);
    create_room(room2, &mut map);
    create_h_tunnel(25, 55, 23, &mut map);

    map
}

/// Create a room within our map
fn create_room(room: Rect, map: &mut Map) {
    // go through the tiles within the rectangle and make them passable
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

/// Create a horizontal tunnel between two rooms
fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    // horizontal tunnel.  `min()` and `map()` are used in case `x1 > x2`
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    // vertical tunnel
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

/// Handle Key inputs from the user
fn handle_keys(tcod: &mut Tcod, player: &mut Object, game: &Game) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    // TODO: Actually handle
    let key = tcod.root.wait_for_keypress(true);
    match key {
        // Alt + Enter: toggle fullscreen
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        // Exit the game when the escape key is pressed
        Key { code: Escape, .. } => return true,
        // movement keys
        Key { code: Up, .. } => player.move_by(0, -1, game),
        Key { code: Down, .. } => player.move_by(0, 1, game),
        Key { code: Left, .. } => player.move_by(-1, 0, game),
        Key { code: Right, .. } => player.move_by(1, 0, game),

        _ => {}
    }
    false
}
