use std::sync::OnceLock;

use tera::Tera;

pub fn tera() -> &'static Tera {
    static TERA: OnceLock<Tera> = OnceLock::new();
    TERA.get_or_init(|| match Tera::new("cli/templates/**") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    })
}
