extern crate syntect;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

#[test]
fn test() {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let set = &ts.themes;
    
    set.into_iter().for_each(|f| {
        println!("{}", f.0);
    });
    

    let syntax = ps.find_syntax_by_extension("xml").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["Solarized (dark)"]);
    let s = "<xml><info name='alex'><age>1</age></info></xml>";
    for line in LinesWithEndings::from(s) {
        let ranges: Vec<(Style, &str)> = h.highlight(line, &ps);
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        println!("{}", escaped);
    }
}
