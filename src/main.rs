use std::path::PathBuf;

#[allow(unused)]
mod ansi;
mod im;
mod task_1;

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
  {c}task1 [path]   {r}Analyze png image file or directory
  {c}h, help        {r}Print help information
"#);
}

fn cmd_task1(path: &str) {
    task_1::run(&PathBuf::from(path));
}
