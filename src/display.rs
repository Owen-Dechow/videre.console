pub struct Display(Vec<String>);

impl Display {
    pub fn new(size: (u16, u16)) -> Self {
        Display(vec![" ".repeat(size.0.into()); size.1.into()])
    }
}
