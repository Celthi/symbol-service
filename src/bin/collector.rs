use anyhow::Result;
use debugid::DebugId;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::exit;
use symbolic::debuginfo::elf::ElfObject;
use symbolic::debuginfo::pe::PeObject;
use symservice::config;
use symservice::crawl_paths;
use symservice::db;
use symservice::debug_entry;
use walkdir::WalkDir;

static OBJECT_FILE_SUFFIX: [&str; 2] = [".dll", ".so"];

fn is_shared_library(s: &str) -> bool {
    OBJECT_FILE_SUFFIX
        .iter()
        .any(|suf| s.to_lowercase().ends_with(suf))
}
fn is_linux_lib(s: &str) -> bool {
    s.to_lowercase().ends_with(".so")
}
fn is_windows_lib(s: &str) -> bool {
    s.to_lowercase().ends_with(".dll")
}

pub struct Collector {
    db_connection: db::DB,
}
impl Collector {
    pub async fn new(host: &str, dbname: &str, user: &str, password: &str) -> Collector {
        match db::DB::new(host, dbname, user, password).await {
            Ok(connect) => Collector {
                db_connection: connect,
            },
            Err(e) => {
                println!("cannot connect to the database: {}.", e);
                exit(3);
            }
        }
    }
    pub async fn run(&mut self) {
        let roots = crawl_paths::read_lib_paths("./config.json").unwrap();
        let mut i = 0;
        // result (filter_map)-> some Option -> filter->map
        let mut debug_entries: Vec<debug_entry::DebugEntry> = Vec::new();
        for root in &roots {
            WalkDir::new(&root.path)
                .follow_links(true)
                .into_iter()
                .filter(|e| {
                    e.is_ok() && is_shared_library(e.as_ref().unwrap().path().to_str().unwrap())
                })
                .map(|e| e.ok().unwrap())
                .for_each(|e| {
                    let path = e.path();
                    if let Some(entry) = self.extract_debug_id(path, &root.note) {
                        debug_entries.push(entry);
                        i += 1;
                    }
                });
        }

        println!("crawled total files: {}", i);
        for entry in debug_entries {
            self.save_debug_id(entry).await;
        }
    }
    async fn save_debug_id(&mut self, entry: debug_entry::DebugEntry) {
        match self.db_connection.insert_one_debug_id(&entry).await {
            Ok(_) => {
                println!("successfully insert to the database{}", entry.debug_id);
            }
            Err(e) => {
                println!("insert into database meets error: {}.", e);
            }
        }
    }
    fn extract_debug_id(&mut self, path: &Path, root: &str) -> Option<debug_entry::DebugEntry> {
        let mut data = Vec::new();
        let mut file = File::open(path).unwrap();
        if let Ok(_res) = file.read_to_end(&mut data) {
            if let Some(debug_id) = get_debug_id(path, &data) {
                return Some(debug_entry::DebugEntry {
                    debug_id,
                    location: path.to_string_lossy().to_string(),
                    note: root.to_owned(),
                });
            }
        } else {
            println!("Unable to read the file {}", path.to_string_lossy());
        }
        None
    }
}

#[tokio::main] // By default, tokio_postgres uses the tokio crate as its runtime.
async fn main() -> Result<()> {
    println!("Hello, world!");
    let mut collector = Collector::new(
        config::POSTGRES_HOST,
        config::POSTGRES_DB_NAME,
        config::POSTGRES_USER,
        config::POSTGRES_PASSWORD,
    )
    .await;
    collector.run().await;
    Ok(())
}
fn get_debug_id(path: &Path, data: &[u8]) -> Option<DebugId> {
    if is_linux_lib(path.to_str().unwrap()) {
        if let Ok(obj) = ElfObject::parse(data) {
            return Some(obj.debug_id());
        }
    }
    if is_windows_lib(path.to_str().unwrap()) {
        if let Ok(obj) = PeObject::parse(data) {
            return Some(obj.debug_id());
        }
    }
    None
}
