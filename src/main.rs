extern crate clap;
extern crate dirs;
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::env::current_dir;
use std::fs;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::ErrorKind::NotFound;
use std::path::PathBuf;

use clap::{App, Arg};
use failure::err_msg;
use failure::Error;

#[derive(Serialize, Deserialize, Debug)]
struct Mark {
    alias: String,
    filepath: String,
}

fn build_listfile_path() -> Result<PathBuf, Error> {
    let prefix = dirs::config_dir().ok_or(err_msg("config directory path is not defined"))?;
    let path = prefix.join("hiraishin/marklist.json");

    Ok(path)
}

fn ensure_dirs_created(path: &PathBuf) -> Result<(), Error> {
    let dirname = path.parent().unwrap(); // It should be have some of parent dirs.
    fs::create_dir_all(dirname)?;

    Ok(())
}

fn load_from_listfile() -> Result<Vec<Mark>, Error> {
    let filepath = build_listfile_path()?;
    ensure_dirs_created(&filepath)?;

    let result = OpenOptions::new().read(true).open(filepath);
    let file = match result {
        Ok(f) => f,
        Err(e) => {
            return if e.kind() == NotFound {
                Ok(vec![])
            } else {
                Err(From::from(e))
            }
        }
    };

    let reader = BufReader::new(file);
    let marks = serde_json::from_reader(reader)?;

    Ok(marks)
}

fn save_to_listfile(marks: &Vec<Mark>) -> Result<(), Error> {
    let filepath = build_listfile_path()?;
    ensure_dirs_created(&filepath)?;

    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(filepath)?;

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &marks)?;

    Ok(())
}

fn add_mark(alias: &str) -> Result<(), Error> {
    let pathbuf = current_dir()?;
    let filepath = pathbuf
        .to_str()
        .ok_or(err_msg("failed to convert current directory into string"))?;

    let mut marks = load_from_listfile()?;
    marks.push(Mark {
        alias: alias.to_owned(),
        filepath: filepath.to_owned(),
    });

    save_to_listfile(&marks)?;

    Ok(())
}

fn remove_mark(alias: &str) -> Result<(), Error> {
    let mut marks = load_from_listfile()?;

    marks.retain(|x| x.alias != alias);

    save_to_listfile(&marks)?;

    Ok(())
}

fn list_marks() -> Result<(), Error> {
    let marks = load_from_listfile()?;

    for x in marks.iter() {
        println!("{:#?}", x);
    }

    Ok(())
}

fn find_mark(alias: &str) -> Result<(), Error> {
    let marks = load_from_listfile()?;

    if let Some(m) = marks.iter().find(|&x| x.alias == alias) {
        println!("{}", m.filepath);
    }

    Ok(())
}

fn output_rc() {
    let rc = r#"alias hiraishin='_hiraishin'
_hiraishin ()
{
    if test "x$1" = "x--look"; then
        proj=$(\hiraishin --find $2);
        if test "x$proj" != "x"; then
            cd $proj || return;
        fi;
    else
        \hiraishin "$@";
    fi
}"#;
    println!("{}", rc);
}

fn main() {
    let app = App::new("Hiraishin")
        .version("0.2.0")
        .author("Sho Minagawa <msh5.global@gmail.com>")
        .arg(
            Arg::with_name("mark")
                .short("m")
                .long("mark")
                .value_name("ALIAS")
                .help("Mark file/directory path")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("unmark")
                .short("d")
                .long("unmark")
                .value_name("ALIAS")
                .help("Unmark file/directory path")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .help("List up marked paths"),
        )
        .arg(
            Arg::with_name("find")
                .short("f")
                .long("find")
                .value_name("ALIAS")
                .help("Look up marked path with alias")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("look")
                .short("k")
                .long("look")
                .help("Change current directory into marked path"),
        )
        .arg(
            Arg::with_name("rc")
                .long("rc")
                .help("Output recommended .bashrc lines"),
        );
    let matcher = app.get_matches();

    if let Some(alias) = matcher.value_of("mark") {
        add_mark(alias).expect("Failed to execute add command");
    }
    if let Some(alias) = matcher.value_of("unmark") {
        remove_mark(alias).expect("Failed to execute unmark command");
    }
    if matcher.is_present("list") {
        list_marks().expect("Failed to execute list command");
    };
    if let Some(alias) = matcher.value_of("find") {
        find_mark(alias).expect("Failed to execute find command");
    };
    if matcher.is_present("look") {
        println!("bashrc installation is required");
    };
    if matcher.is_present("rc") {
        output_rc();
    };
}
