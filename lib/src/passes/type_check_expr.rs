enum Polytype {
    Monotype(Monotype),
    Quantifier(Vec<String>, Box<Polytype>),
}

enum Monotype {
    Variable(String),
    Application(Box<Monotype>, Box<Monotype>),
}

