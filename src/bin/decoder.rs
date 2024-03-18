use gif_parser::gif;
use std::env;
use std::result;

type Result<T> = result::Result<T, ()>;

fn main() -> Result<()> {
    let mut args = env::args();
    println!("Running {program}", program = args.next().unwrap());
    match args.next() {
        Some(file_path) => {
            let gif_sign: gif::GifSignature = gif::Gif::decode(&file_path).signature;
            println!("INFO: loaded gif version {gif_sign:?} from file {file_path}");
        }

        None => {
            let file_path = "res/stars.gif";
            println!("WARN: No file path provided using default path: {file_path}");
            let gif_sign: gif::GifSignature = gif::Gif::decode(file_path).signature;
            println!("INFO: loaded gif version {gif_sign:?} from file {file_path}");
        }
    };
    Ok(())
}
