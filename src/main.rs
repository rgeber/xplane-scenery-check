use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use clap::{arg, Parser};
use glob::glob;
use rayon::prelude::*;
use rayon::iter::ParallelBridge;

#[derive(Debug)]
struct FileErrorData {
    source_path: String,
    target_path: String
}

#[derive(Debug)]
enum FileError {
    DoesNotExist(FileErrorData),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Check your X-Plane scenery for errors.", arg_required_else_help = true)]
struct Args {
    /// Path to the x-plane root directory.
    #[arg(short('x'), long("xplane"), required = true)]
    xplane_root_dir: String,
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn check_base_tex_nowrap(line: String, path: String) -> Result<(), FileError> {
    let b: Vec<&str> = line.as_str().split(' ').collect();
    let p = Path::new(path.as_str()).parent().unwrap().join(b[1]);
    match p.exists() {
        true => Ok(()),
        false => Err(FileError::DoesNotExist(FileErrorData{
            source_path: path,
            target_path: p.display().to_string()
        }))
    }
}

fn main() {

    let args = Args::parse();

    let xplane_root_dir = Path::new(&args.xplane_root_dir);
    xplane_root_dir.exists().then(|| true).or_else(|| panic!("Root path does not exist."));

    let xplane_custom_scenery_dir = xplane_root_dir.join("Custom Scenery");
    xplane_custom_scenery_dir.exists().then(|| true).or_else(|| panic!("Unable to find Custom Scenery directory."));

    let path_as_string = xplane_custom_scenery_dir.display().to_string();
    let glob_pattern = format!("{}/**/*.ter", path_as_string);

    glob(glob_pattern.as_str()).expect("Failed to read glob pattern").into_iter().par_bridge().for_each(|gr| {
        let path = gr.unwrap().display().to_string();

        if let Ok(lines) = read_lines(&path) {
            for  line in lines {
                if let Ok(value) = line {
                    if value.starts_with("BASE_TEX_NOWRAP") {
                        match check_base_tex_nowrap(value, path.clone()) {
                            Ok(()) => (),
                            Err(e) => match e {
                                FileError::DoesNotExist(error_data) => {
                                    eprintln!("ERROR in `{}`: Unable to find `{}`.", error_data.source_path, error_data.target_path)
                                }
                            }
                        };
                    }
                }
            }
        }
    });

    println!("\n\n   Aaaaaand we're done here.");

}