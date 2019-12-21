use rand::Rng;
use std::cmp;
use tcod::colors::*;
use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

/*********  CONSTANTS  **********/
/// Player
const PLAYER: usize = 0;
/// Actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
/// size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;
/// Other settings
const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum
/// Room settings
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;
// Monster Settings
const MAX_ROOM_MONSTERS: i32 = 3;
// Lighting and FOV
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // default FOV algorithm
const FOV_LIGHT_WALLS: bool = true; // light walls or not
const TORCH_RADIUS: i32 = 10;
/// Colors
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

/*********  STRUCTURES  *********/
struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap,
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
    explored: bool,
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

    let mut tcod = Tcod {
        root,
        con: Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    };

    // Initilize Player Properties
    let player = Object::new(25, 23, '@', WHITE);
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', YELLOW);

    // List of objects in the game, currently player, npc
    let mut objects = vec![player];
    let mut game = Game {
        map: make_map(&mut objects),
    };

    // populate the FOV map, according to the generated map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }
    // force FOV "recompute" first time through the game loop
    let mut previous_player_position = (-1, -1);

    while !tcod.root.window_closed() {
        // clear the screen of the previous frame
        tcod.con.clear();
        // handle the updating of the view port
        // render the screen
        let fov_recompute = previous_player_position != objects[PLAYER].pos();
        render_all(&mut tcod, &mut game, &objects, fov_recompute);

        tcod.root.flush();

        // Key handleing w/ exit
        let player = &mut objects[PLAYER];
        previous_player_position = (player.x, player.y);
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

    /// return the position of the object
    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    /// set the position of the object
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

/// Functions for the tile structure
impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            explored: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            explored: false,
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

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        // returns tru if this rectangle intersects with another one
        (self.x1 <= other.x2)
            && (self.x2 > other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
}

/********** GENERIC FUNCTIONS ***********/

/// render all of the things
fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
    if fov_recompute {
        // recompute FOV if needed (the player moved or something)
        let player = &objects[PLAYER];
        tcod.fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    // Draw all of our objects on the screen
    for object in objects {
        if tcod.fov.is_in_fov(object.x, object.y) {
            object.draw(&mut tcod.con);
        }
    }

    // Draw all our tiles onto the screen
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = game.map[x as usize][y as usize].block_sight;
            let color = match (visible, wall) {
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };
            let explored = &mut game.map[x as usize][y as usize].explored;
            if visible {
                // since it's visible, explore it
                *explored = true;
            }
            if *explored {
                tcod.con
                    .set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }
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
fn make_map(objects: &mut Vec<Object>) -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    // create the rooms
    let mut rooms = vec![];
    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        // random positions without going off the bounds of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);

        // run through and check if any others intersect
        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            // valid room with no intersections

            // "paint" it to the map's tiles
            create_room(new_room, &mut map);

            // "fill" it with monsters
            place_objects(new_room, objects);

            // center coordinates fo the new room, will be useful later
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                // this is the first room, where the player starts at
                objects[PLAYER].set_pos(new_x, new_y);
            } else {
                // all rooms after the first:
                // connect it to the previous with a tunnel

                // center coords of the previous room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                // toss a coin
                if rand::random() {
                    // first move horizontally, then vert
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    // first move vert, then horiz
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }

            // append the new room to the list
            rooms.push(new_room);
        }
    }

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

/// Create a vertical tunnel between two rooms
fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    // vertical tunnel
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

/// Place some monsters in a room
fn place_objects(room: Rect, objects: &mut Vec<Object>) {
    // chose a random number of monsters
    let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

    for _ in 0..num_monsters {
        // chose random spot for this monster
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        let mut monster = if rand::random::<f32>() < 0.8 {
            // 80% chance of getting an orc
            // create an orc
            Object::new(x, y, 'o', DESATURATED_GREEN)
        } else {
            Object::new(x, y, 'T', DARKER_GREEN)
        };

        objects.push(monster);
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
