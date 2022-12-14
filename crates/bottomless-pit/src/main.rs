mod game;
mod settings;
mod player;

use settings::Settings;
use game::Game;

fn main() {
    let settings = Settings::load_from_file();

    let settings = match settings {
        Ok(settings) => {
            println!("settings loaded succsesfully");
            settings
        },
        Err(e) => {
            println!("settings could not be loaded {}", e);
            Settings::default()
        }
    };

    let (length, height) = settings.get_resoultion();

    let(mut rl, thread) = raylib::init()
        .size(length as i32, height as i32)
        .title("cheesed to meet u")
        .resizable()
        .build();
    
    rl.set_target_fps(30);
    rl.set_exit_key(None);

    let mut game = Game::new(settings);

    while !game.should_close(&rl) {
        game.update(&mut rl, &thread);
        let d_handle = rl.begin_drawing(&thread);
        game.draw(d_handle);
    }
}