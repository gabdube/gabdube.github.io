use std::collections::{HashMap, HashSet};
use std::fs::{read, read_to_string};
use std::sync::{Mutex, Arc, mpsc::{Sender, Receiver, channel}};
use std::time::Duration;

use rouille::{websocket, Response, Request};

#[derive(Clone)]
enum FileType {
    Text(String),
    Bin(Vec<u8>)
}

impl FileType {
    fn text(self) -> String {
        match self {
            Self::Text(value) => value,
            Self::Bin(_) => "ERROR. FILE IS BINARY.".to_string()
        }
    }

    fn bin(self) -> Vec<u8> {
        match self {
            Self::Bin(value) => value,
            Self::Text(value) => value.as_bytes().to_vec(),
        }
    }
}

#[derive(Default)]
struct AssetsCollection {
    files: HashMap<String, FileType>,

    // One sender for each active websocket collection
    // The web-path of the changed item is sent to the client
    // If the app needs to reload it, it will send a GET request to the server
    reloaded_listeners: Vec<Sender<String>>,   
}

impl AssetsCollection {
    pub fn get_cloned(&self, web_path: &str) -> Option<FileType> {
        self.files.get(web_path).cloned()
    }

    pub fn get_mut(&mut self, web_path: &str) -> Option<&mut FileType> {
        self.files.get_mut(web_path)
    }

    pub fn listen(&mut self) -> Receiver<String> {
        let (sender, receiver) = channel();
        self.reloaded_listeners.push(sender);
        receiver
    }
}

type SharedAssetsCollection = Arc<Mutex<AssetsCollection>>;

const ASSETS_EXTENSIONS_TO_RELOAD: &[&str] = &["", "html", "js", "css", "svg", "wasm", "glsl", "png", "csv", "ttf"];

fn load_text_file(files: &mut HashMap<String, FileType>, web_path: &str, local_path: &str) {
    match read_to_string(local_path) {
        Ok(content) => { files.insert(web_path.to_string(), FileType::Text(content)); },
        _ => { panic!("File not found {:?}", local_path); }
    }
}

fn load_bin_file(files: &mut HashMap<String, FileType>, web_path: &str, local_path: &str) {
    match read(local_path) {
        Ok(content) => { files.insert(web_path.to_string(), FileType::Bin(content)); },
        _ => { panic!("File not found {:?}", local_path); }
    }
}

fn preload_all_by_extensions(files: &mut HashMap<String, FileType>, extension: &str, text: bool) {
    let pattern = format!("./articles/**/*.{}", extension);
    for file in glob::glob(&pattern).unwrap().filter_map(Result::ok) {
        if let Some(local_path) = file.to_str() {
            let web_path = format!("/{}", local_path).replace("\\", "/");
            if web_path.matches("/build/").next().is_some() || web_path.matches("src/").next().is_some() {
                continue; // Skip build & src folder
            }

            if text {
                load_text_file(files, &web_path, local_path);
            } else {
                load_bin_file(files, &web_path, local_path);
            }
        }
    }
}

fn preload_files() -> SharedAssetsCollection {
    let mut collection = AssetsCollection::default();

    let f = &mut collection.files;
    load_text_file(f, "/", "index.html");
    load_text_file(f, "/index.html", "index.html");
    load_text_file(f, "/styles.css", "styles.css");
    load_text_file(f, "/favico.svg", "favico.svg");
    load_bin_file(f, "/FiraCode-Regular.ttf", "FiraCode-Regular.ttf");

    preload_all_by_extensions(f, "html", true);
    preload_all_by_extensions(f, "js", true);
    preload_all_by_extensions(f, "glsl", true);
    preload_all_by_extensions(f, "csv", true);
    preload_all_by_extensions(f, "wasm", false);
    preload_all_by_extensions(f, "png", false);

    Arc::new(Mutex::new(collection))
}

fn must_reload_file(event: &notify::event::Event) -> bool {
    match event.kind {
        notify::EventKind::Modify(_) => {},
        _ => { return false; }
    }

    let path = match event.paths.first() {
        Some(path) => path,
        None => { return false; }
    };

    let extension = path.extension().and_then(|ext| ext.to_str() ).unwrap_or("");
    if !ASSETS_EXTENSIONS_TO_RELOAD.iter().any(|&ext| extension == ext ) {
        return false;
    }

    true
}

#[cfg(feature="watch")]
fn watch_files(assets: &SharedAssetsCollection) {
    use std::sync::mpsc;
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

    const WAIT: ::std::time::Duration = ::std::time::Duration::from_millis(200);

    let assets_guard = Arc::clone(assets);
    ::std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();

        let base_path = ::std::path::Path::new(".").canonicalize().unwrap();
        let base_path_str = base_path.to_str().unwrap();

        let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
        watcher.watch(base_path.as_ref(), RecursiveMode::Recursive).unwrap();

        let mut accumulate: HashSet<(String, String)> = HashSet::default();

        loop {
            while let Ok(Ok(event)) = rx.recv_timeout(WAIT) {
                if must_reload_file(&event) {
                    let local_path = event.paths.first()
                        .and_then(|p| p.to_str() )
                        .map(|p| p.to_string() )
                        .unwrap();

                    let web_path = local_path
                        .replace(base_path_str, "")
                        .replace("\\", "/");

                    accumulate.insert((local_path, web_path));
                }
            }

            if accumulate.len() > 0 {
                let mut assets = assets_guard.lock().unwrap();
                for (local_path, web_path) in accumulate.iter() {
                    if let Some(source) = assets.get_mut(&web_path) {
                        match source {
                            FileType::Text(value) => { *value = read_to_string(local_path).unwrap_or("".to_string()); }
                            FileType::Bin(value) => { *value = read(local_path).unwrap_or(Vec::new()); }
                        }

                        // Retain lets us clean the closed connections
                        assets.reloaded_listeners.retain(|sender| {
                            sender.send(web_path.clone()).is_ok()
                        });
                    }
                }

                accumulate.clear();
            }
        }

    });
}

#[cfg(feature="watch")]
fn handle_websocket(assets: &SharedAssetsCollection, request: &Request) -> Option<Response> {
    if request.header("Upgrade") != Some("websocket") {
        return None;
    }

    let (response, websocket) = match websocket::start::<&str>(request, None) {
        Ok(r) => r,
        Err(err) => {
            let json = rouille::try_or_400::ErrJson::from_err(&err);
            return Some(Response::json(&json).with_status_code(400));
        }
    };

    let assets_guard = Arc::clone(&assets);

    ::std::thread::spawn(move || {
        let initial_connection_timeout = Duration::from_millis(1000);
        let mut connection = match websocket.recv_timeout(initial_connection_timeout) {
            Ok(connection) => connection,
            Err(_) => { return; }
        };

        println!("New websocket connection");

        // Adds the new connection into the watches list
        let assets_to_reload = {
            assets_guard.lock().unwrap().listen()
        };
        
        let tick = Duration::from_millis(100);
        'outer: loop {
            if connection.is_closed() {
                break;
            }

            while let Ok(path) = assets_to_reload.try_recv() {
                let cmd = format!("{{ \"name\": \"FILE_CHANGED\", \"data\": {:?} }}", path);
                if let Err(_) = connection.send_text(&cmd) {
                    break 'outer;
                }
            }

            ::std::thread::sleep(tick);
        }

        println!("Websocket collection closed");
    });

    Some(response)
}

#[cfg(not(feature="watch"))]
fn watch_files(assets: &AssetsCollSharedAssetsCollectionction) {}

#[cfg(not(feature="watch"))]
fn handle_websocket(_assets: &SharedAssetsCollection, request: &Requestt) -> Option<Response> {
    if request.header("Upgrade") != Some("websocket") {
        None
    } else {
        let error = "{\"error\": \"websocket server feature disabled\"}";
        let response = Response::from_data("application/json", error.as_bytes());
        Some(response)
    }
}

fn response_from_url(url: &str, data: FileType) -> Response {
    let path_extension = ::std::path::Path::new(url).extension().and_then(|ext| ext.to_str() ).unwrap_or("");
    match path_extension {
        "" | "html" => Response::html(data.text()),
        "svg"       => Response::svg(data.text()),
        "css"       => Response::from_data("text/css; charset=utf-8", data.bin()),
        "js"        => Response::from_data("text/javascript; charset=utf-8", data.bin()),
        "wasm"      => Response::from_data("application/wasm", data.bin()),
        _           => Response::from_data("application/octet-stream", data.bin())
    }
}

fn get_asset(assets: &SharedAssetsCollection, url: &str) -> Option<FileType> {
    assets.lock().unwrap().get_cloned(url)
}

fn main() {
    let assets = preload_files();

    watch_files(&assets);

    rouille::start_server("localhost:8001", move |request| {
        match request.method() {
            "GET" => {
                if let Some(response) = handle_websocket(&assets, &request) {
                    return response;
                }

                let url = request.url();
                match get_asset(&assets, &url) {
                    Some(data) => response_from_url(&url, data),
                    None => Response::empty_404()
                }
            },

            _ => Response::empty_204()
        }
    });
}
