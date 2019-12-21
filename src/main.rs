use tcod::colors::*;
use tcod::console::*;

/*********  CONSTANTS  **********/
/// Actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
/// Other settings
const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum

/*********  STRUCTURES  *********/
struct Tcod {
    root: Root,
    con: Offscreen,
}

/// This is a generic object: the player, a monster, an item, the stairs...
/// It's always represented by a charactor on the screen.
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
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
    let player = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', WHITE);
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', YELLOW);

    // List of objects in the game, currently player, npc
    let mut objects = [player, npc];

    while !tcod.root.window_closed() {
        tcod.con.clear(); // clean console
        for object in &objects {
            object.draw(&mut tcod.con);
        }
        tcod.root.flush(); // clean root console to write to
                           // bit the contents of "con" onto the root console and present it
        blit(
            &tcod.con,
            (0, 0),
            (SCREEN_WIDTH, SCREEN_HEIGHT),
            &mut tcod.root,
            (0, 0),
            1.0,
            1.0,
        );
        // Key handleing w/ exit
        let player = &mut objects[0];
        let exit = handle_keys(&mut tcod, player);
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
    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }

    /// set the color and then draw the character that represents this object at it's
    /// position on the screen
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

/// Handle Key inputs from the user
fn handle_keys(tcod: &mut Tcod, player: &mut Object) -> bool {
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
        Key { code: Up, .. } => player.move_by(0, -1),
        Key { code: Down, .. } => player.move_by(0, 1),
        Key { code: Left, .. } => player.move_by(-1, 0),
        Key { code: Right, .. } => player.move_by(1, 0),

        _ => {}
    }
    false
}
