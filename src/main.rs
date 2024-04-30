use std::path::PathBuf;

#[allow(unused)]
mod ansi;
mod im;
mod task_1;
mod task_2;
mod task_3;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = if let Some(arg) = args.get(0) {
        arg
    } else {
        eprintln!("\ncommand is missing, use `image_alg help` to learn the usage\n");
        return;
    };

    match command.as_str() {
        "h" | "help" => {
            cmd_help();
        }
        "task1" => {
            if let Some(file) = args.get(1) {
                cmd_task1(file);
            } else {
                eprintln!("\nmissing path to image or directory\n");
            }
        }
        "task2" => {
            if let Some(file) = args.get(1) {
                if let Some(width_downsize) = args.get(2) {
                    cmd_task2(
                        file,
                        width_downsize
                            .parse::<u32>()
                            .expect("parsed width_downsize integer"),
                    );
                } else {
                    eprintln!("\nmissing width_downsize amount\n");
                }
            } else {
                eprintln!("\nmissing path to image or directory\n");
            }
        }
        "task3" => {
            if let Some(file) = args.get(1) {
                if let Some(intencity_levels) = args.get(2) {
                    cmd_task3(
                        file,
                        intencity_levels
                            .parse::<u32>()
                            .expect("parsed intencity_levels integer"),
                    );
                } else {
                    eprintln!("\nmissing intencity_levels count\n");
                }
            } else {
                eprintln!("\nmissing path to image or directory\n");
            }
        }
        _ => {
            eprintln!(
                "\nunknown command `{}`, use `image_alg help` to learn the usage\n",
                command
            );
        }
    }
}

fn cmd_help() {
    let g = ansi::GREEN_BOLD;
    let c = ansi::CYAN_BOLD;
    let r = ansi::RESET;

    #[rustfmt::skip]
        println!(
r#"
{g}Usage:
  {c}image_alg <command>

{g}Commands:
  {c}task1   [path]                {r}Analyze object bounds
  {c}task2   [path] [width amount] {r}Downsize content aware horizontal
  {c}task3   [path] [int levels]   {r}Perform block & rotation & intensity transformations
  {c}h, help                       {r}Print help information
"#);
}

fn cmd_task1(path: &str) {
    task_1::run(&PathBuf::from(path));
}

fn cmd_task2(path: &str, width_downsize: u32) {
    task_2::run(&PathBuf::from(path), width_downsize);
}

fn cmd_task3(path: &str, intensity_levels: u32) {
    task_3::run(&PathBuf::from(path), intensity_levels);
}
