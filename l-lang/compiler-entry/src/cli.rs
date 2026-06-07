use clap::Parser;

/** Very basic mips compiler for dog, a custom c-like language
 Compiled files are stored in the same directory as the source as <file.asm>
**/
#[derive(Parser, Debug)]
#[command(
    version = "4.2.0 by ElEmDee",
    about =
    "
Very basic mips compiler for l-lang, a custom c-like language
Compiled files are stored in the same directory as the source as <file.asm>",
    long_about = None
)]
pub struct Args {
    /// Input file, must end in .l extention
    #[arg(value_name = "file.l")]
    pub input_file: String,

    /// Print Abstract Syntax Tree to stdout
    #[arg(long)]
    pub ast: bool,

    /// Print Semantic Analysis Tree to stdout
    #[arg(long)]
    pub sat: bool,

    /// Print the IR representation to stdout
    #[arg(long)]
    pub ir: bool,

    /// Print tokens to stdout
    #[arg(long)]
    pub tokens: bool,

    /// Name of the output asm file
    #[arg(short = 'S', default_value = "out.asm")]
    pub output: String,
}
