mod shared;
mod helpers;
mod pack_sprites;
mod pack_terrain;
mod convert_tilemap;

fn command_name() -> Option<String> {
    ::std::env::args().skip(1).next()
}

fn main() {
    let cmd = match command_name() {
        Some(cmd) => cmd,
        None => {
            println!("Missing command. Usage:");
            println!("cargo run --release -p tools -- *command_name* [*command_arguments*]");
            println!("cargo run --release -p tools -- pack_sprites -i sprites.csv");
            return;
        }
    };

    match cmd.as_str() {
        "pack-sprites" => { pack_sprites::run(); },
        "pack-terrain" => { pack_terrain::run(); },
        "convert-tilemap" => { convert_tilemap::run(); },
        _ => {
            eprintln!("Unknown command name {:?}", cmd);
        }
    }
}
