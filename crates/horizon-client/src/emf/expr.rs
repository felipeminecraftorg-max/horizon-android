//! The animation expressions of Entity Model Features (EMF) and OptiFine's Custom Entity Models
//! (`.jem` files' `animations`): `target = expression` lines over the model's parts (`head.rx`),
//! the entity's values (`limb_swing`, `is_on_ground`) and its own variables (`var.name`), which
//! keep their value from one frame to the next.
//!
//! Numbers are `f64`; a comparison or a flag is 1 or 0, and anything but 0 is true. A name the
//! game does not know reads 0, and is listed in [`Program::unknown`].

/// A channel of a model part: its pivot (`tx`, `ty`, `tz`, in pixels), rotation (`rx`, `ry`, `rz`,
/// radians) and scale (`sx`, `sy`, `sz`), the order of a part's pose.
pub const CHANNELS: [&str; 9] = ["tx", "ty", "tz", "rx", "ry", "rz", "sx", "sy", "sz"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Func {
    If,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    ToRad,
    ToDeg,
    Min,
    Max,
    Clamp,
    Abs,
    Floor,
    Ceil,
    Round,
    Exp,
    Frac,
    Log,
    Pow,
    Sqrt,
    Signum,
    Fmod,
    Lerp,
    Between,
    Equals,
    In,
    Random,
    RandomB,
    WrapDeg,
    WrapRad,
    DegDiff,
    RadDiff,
    /// `print(id, every, x)`, `printb`: the value itself.
    Print,
}

impl Func {
    fn named(name: &str) -> Option<Self> {
        Some(match name {
            "if" | "ifb" => Self::If,
            "sin" => Self::Sin,
            "cos" => Self::Cos,
            "tan" => Self::Tan,
            "asin" => Self::Asin,
            "acos" => Self::Acos,
            "atan" => Self::Atan,
            "atan2" => Self::Atan2,
            "torad" => Self::ToRad,
            "todeg" => Self::ToDeg,
            "min" => Self::Min,
            "max" => Self::Max,
            "clamp" => Self::Clamp,
            "abs" => Self::Abs,
            "floor" => Self::Floor,
            "ceil" => Self::Ceil,
            "round" => Self::Round,
            "exp" => Self::Exp,
            "frac" => Self::Frac,
            "log" => Self::Log,
            "pow" => Self::Pow,
            "sqrt" => Self::Sqrt,
            "signum" => Self::Signum,
            "fmod" => Self::Fmod,
            "lerp" => Self::Lerp,
            "between" => Self::Between,
            "equals" => Self::Equals,
            "in" => Self::In,
            "random" => Self::Random,
            "randomb" => Self::RandomB,
            "wrapdeg" => Self::WrapDeg,
            "wraprad" => Self::WrapRad,
            "degdiff" => Self::DegDiff,
            "raddiff" => Self::RadDiff,
            "print" | "printb" => Self::Print,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f64),
    /// One of the entity's own variables, by slot.
    Var(usize),
    /// A model part's channel: (part slot, channel).
    Part(usize, u8),
    /// One of the entity's values, by slot.
    Input(usize),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Call(Func, Vec<Expr>),
}

/// Where a line's value goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Target {
    Var(usize),
    Part(usize, u8),
    /// A value the game has no use for (`render.shadow_size`, a part's `visible`).
    Ignored,
}

/// Names a program reads: what the game resolves.
pub trait Names {
    /// The slot of a model part (`head`, `left_arm`).
    fn part(&mut self, name: &str) -> Option<usize>;
    /// The slot of an entity value (`limb_swing`).
    fn input(&self, name: &str) -> Option<usize>;
}

/// A model's animations, compiled.
#[derive(Clone, Debug, Default)]
pub struct Program {
    pub lines: Vec<(Target, Expr)>,
    /// The entity variables, by slot (`var.name`, `varb.name` and the global ones share names).
    pub vars: Vec<String>,
    /// Names nobody gives a value: they read 0.
    pub unknown: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Num(f64),
    Name(String),
    Op(&'static str),
    Open,
    Close,
    Comma,
}

fn tokens(text: &str) -> Result<Vec<Token>, String> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || (c == b'.' && b.get(i + 1).is_some_and(u8::is_ascii_digit))
        {
            let start = i;
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
                i += 1;
            }
            if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
                let mut j = i + 1;
                if j < b.len() && (b[j] == b'-' || b[j] == b'+') {
                    j += 1;
                }
                if j < b.len() && b[j].is_ascii_digit() {
                    i = j;
                    while i < b.len() && b[i].is_ascii_digit() {
                        i += 1;
                    }
                }
            }
            let s = &text[start..i];
            out.push(Token::Num(
                s.parse().map_err(|_| format!("not a number: {s}"))?,
            ));
        } else if c.is_ascii_alphabetic() || c == b'_' {
            let start = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.') {
                i += 1;
            }
            out.push(Token::Name(text[start..i].to_owned()));
        } else {
            let two = text.get(i..i + 2).unwrap_or_default();
            let op = ["&&", "||", "<=", ">=", "==", "!="]
                .into_iter()
                .find(|o| *o == two);
            if let Some(op) = op {
                out.push(Token::Op(op));
                i += 2;
                continue;
            }
            out.push(match c {
                b'(' => Token::Open,
                b')' => Token::Close,
                b',' => Token::Comma,
                b'+' => Token::Op("+"),
                b'-' => Token::Op("-"),
                b'*' => Token::Op("*"),
                b'/' => Token::Op("/"),
                b'%' => Token::Op("%"),
                b'<' => Token::Op("<"),
                b'>' => Token::Op(">"),
                b'!' => Token::Op("!"),
                // `=` alone compares, as OptiFine reads it.
                b'=' => Token::Op("=="),
                _ => return Err(format!("unexpected {:?}", c as char)),
            });
            i += 1;
        }
    }
    Ok(out)
}

/// The binding power of a binary operator: `||` lowest, then `&&`, equality, order, sums and
/// products.
fn power(op: &str) -> Option<(u8, BinOp)> {
    Some(match op {
        "||" => (1, BinOp::Or),
        "&&" => (2, BinOp::And),
        "==" => (3, BinOp::Eq),
        "!=" => (3, BinOp::Ne),
        "<" => (4, BinOp::Lt),
        "<=" => (4, BinOp::Le),
        ">" => (4, BinOp::Gt),
        ">=" => (4, BinOp::Ge),
        "+" => (5, BinOp::Add),
        "-" => (5, BinOp::Sub),
        "*" => (6, BinOp::Mul),
        "/" => (6, BinOp::Div),
        "%" => (6, BinOp::Mod),
        _ => return None,
    })
}

struct Parser<'a, N: Names> {
    tokens: Vec<Token>,
    at: usize,
    program: &'a mut Program,
    names: &'a mut N,
}

impl<N: Names> Parser<'_, N> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }
    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.at).cloned();
        self.at += 1;
        t
    }
    fn expr(&mut self, min: u8) -> Result<Expr, String> {
        let mut left = self.unary()?;
        while let Some(Token::Op(op)) = self.peek() {
            let Some((p, bin)) = power(op) else {
                break;
            };
            if p < min {
                break;
            }
            self.at += 1;
            let right = self.expr(p + 1)?;
            left = Expr::Bin(bin, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::Op("-")) => {
                self.at += 1;
                Ok(Expr::Neg(Box::new(self.unary()?)))
            }
            Some(Token::Op("+")) => {
                self.at += 1;
                self.unary()
            }
            Some(Token::Op("!")) => {
                self.at += 1;
                Ok(Expr::Not(Box::new(self.unary()?)))
            }
            _ => self.primary(),
        }
    }
    fn primary(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Token::Num(n)) => Ok(Expr::Num(n)),
            Some(Token::Open) => {
                let e = self.expr(0)?;
                match self.next() {
                    Some(Token::Close) => Ok(e),
                    _ => Err("missing )".into()),
                }
            }
            Some(Token::Name(name)) => {
                if self.peek() == Some(&Token::Open) {
                    self.at += 1;
                    let mut args = Vec::new();
                    if self.peek() == Some(&Token::Close) {
                        self.at += 1;
                    } else {
                        loop {
                            args.push(self.expr(0)?);
                            match self.next() {
                                Some(Token::Comma) => continue,
                                Some(Token::Close) => break,
                                _ => return Err(format!("{name}( is not closed")),
                            }
                        }
                    }
                    let func = Func::named(&name).ok_or_else(|| format!("no function {name}"))?;
                    Ok(Expr::Call(func, args))
                } else {
                    Ok(self.name(&name))
                }
            }
            other => Err(format!("unexpected {other:?}")),
        }
    }
    /// A name read in an expression.
    fn name(&mut self, name: &str) -> Expr {
        match name {
            "pi" => return Expr::Num(std::f64::consts::PI),
            "e" => return Expr::Num(std::f64::consts::E),
            "true" => return Expr::Num(1.0),
            "false" => return Expr::Num(0.0),
            _ => {}
        }
        match resolve(name, self.program, self.names) {
            Target::Var(slot) => Expr::Var(slot),
            Target::Part(part, ch) => Expr::Part(part, ch),
            Target::Ignored => match self.names.input(name) {
                Some(slot) => Expr::Input(slot),
                None => {
                    if !self.program.unknown.iter().any(|u| u == name) {
                        self.program.unknown.push(name.to_owned());
                    }
                    Expr::Num(0.0)
                }
            },
        }
    }
}

/// A variable or a part's channel by its name; anything else is `Ignored` (an input, a render
/// value, an unknown name).
fn resolve<N: Names>(name: &str, program: &mut Program, names: &mut N) -> Target {
    let var = ["var.", "varb.", "global_var.", "global_varb."]
        .iter()
        .find_map(|p| name.strip_prefix(p));
    if let Some(v) = var {
        let slot = match program.vars.iter().position(|n| n == v) {
            Some(s) => s,
            None => {
                program.vars.push(v.to_owned());
                program.vars.len() - 1
            }
        };
        return Target::Var(slot);
    }
    if let Some((part, ch)) = name.rsplit_once('.')
        && let Some(c) = CHANNELS.iter().position(|c| *c == ch)
        && let Some(slot) = names.part(part)
    {
        return Target::Part(slot, c as u8);
    }
    Target::Ignored
}

impl Program {
    /// Adds the lines of an `animations` object, in order. A line that does not parse is left out,
    /// with why in the returned list.
    pub fn add<'a, N: Names>(
        &mut self,
        lines: impl IntoIterator<Item = (&'a str, &'a str)>,
        names: &mut N,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        for (target, text) in lines {
            let to = resolve(target.trim(), self, names);
            let parsed = tokens(text).and_then(|tokens| {
                let mut p = Parser {
                    tokens,
                    at: 0,
                    program: self,
                    names,
                };
                let e = p.expr(0)?;
                match p.peek() {
                    None => Ok(e),
                    Some(t) => Err(format!("unexpected {t:?}")),
                }
            });
            match parsed {
                Ok(e) => self.lines.push((to, e)),
                Err(why) => errors.push(format!("{target}: {why}")),
            }
        }
        errors
    }

    /// Runs every line in order over the entity's variables (`vars`, grown to the program's),
    /// its values and the parts' poses.
    pub fn run(&self, vars: &mut Vec<f64>, inputs: &[f64], parts: &mut [[f32; 9]]) {
        if vars.len() < self.vars.len() {
            vars.resize(self.vars.len(), 0.0);
        }
        for (target, e) in &self.lines {
            let v = eval(e, vars, inputs, parts);
            // A NaN or an infinity would hide the part or spoil a variable for good.
            let v = if v.is_finite() { v } else { 0.0 };
            match *target {
                Target::Var(s) => vars[s] = v,
                Target::Part(p, c) => {
                    if let Some(pose) = parts.get_mut(p) {
                        pose[usize::from(c)] = v as f32;
                    }
                }
                Target::Ignored => {}
            }
        }
    }
}

fn truth(v: f64) -> bool {
    v != 0.0
}

fn flag(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

/// A number from `seed`, the same every time: 0 up to 1.
fn seeded(seed: f64) -> f64 {
    let mut x = seed.to_bits() ^ 0x9E37_79B9_7F4A_7C15;
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x >> 11) as f64 / (1u64 << 53) as f64
}

fn wrap(v: f64, half: f64) -> f64 {
    let full = half * 2.0;
    let w = (v + half).rem_euclid(full) - half;
    if w == -half { half } else { w }
}

pub fn eval(e: &Expr, vars: &[f64], inputs: &[f64], parts: &[[f32; 9]]) -> f64 {
    let ev = |x: &Expr| eval(x, vars, inputs, parts);
    match e {
        Expr::Num(n) => *n,
        Expr::Var(s) => vars.get(*s).copied().unwrap_or(0.0),
        Expr::Input(s) => inputs.get(*s).copied().unwrap_or(0.0),
        Expr::Part(p, c) => parts
            .get(*p)
            .map_or(0.0, |pose| f64::from(pose[usize::from(*c)])),
        Expr::Neg(x) => -ev(x),
        Expr::Not(x) => flag(!truth(ev(x))),
        Expr::Bin(op, a, b) => match op {
            BinOp::And => flag(truth(ev(a)) && truth(ev(b))),
            BinOp::Or => flag(truth(ev(a)) || truth(ev(b))),
            _ => {
                let (a, b) = (ev(a), ev(b));
                match op {
                    BinOp::Add => a + b,
                    BinOp::Sub => a - b,
                    BinOp::Mul => a * b,
                    // A division by zero is 0 here: a pack's first frame divides by a frame
                    // time not measured yet, and an infinity would spoil its variables for good.
                    BinOp::Div if b == 0.0 => 0.0,
                    BinOp::Div => a / b,
                    BinOp::Mod if b == 0.0 => 0.0,
                    BinOp::Mod => a % b,
                    BinOp::Lt => flag(a < b),
                    BinOp::Le => flag(a <= b),
                    BinOp::Gt => flag(a > b),
                    BinOp::Ge => flag(a >= b),
                    BinOp::Eq => flag(a == b),
                    BinOp::Ne => flag(a != b),
                    BinOp::And | BinOp::Or => unreachable!(),
                }
            }
        },
        Expr::Call(f, args) => {
            let arg = |i: usize| args.get(i).map_or(0.0, ev);
            match f {
                // if(condition, value, [condition, value...], otherwise): only what is picked runs.
                Func::If => {
                    let mut i = 0;
                    while i + 1 < args.len() {
                        if truth(ev(&args[i])) {
                            return ev(&args[i + 1]);
                        }
                        i += 2;
                    }
                    if i < args.len() { ev(&args[i]) } else { 0.0 }
                }
                Func::Sin => arg(0).sin(),
                Func::Cos => arg(0).cos(),
                Func::Tan => arg(0).tan(),
                Func::Asin => arg(0).asin(),
                Func::Acos => arg(0).acos(),
                Func::Atan => arg(0).atan(),
                Func::Atan2 => arg(0).atan2(arg(1)),
                Func::ToRad => arg(0).to_radians(),
                Func::ToDeg => arg(0).to_degrees(),
                Func::Min => args.iter().map(ev).fold(f64::INFINITY, f64::min),
                Func::Max => args.iter().map(ev).fold(f64::NEG_INFINITY, f64::max),
                Func::Clamp => {
                    let (x, lo, hi) = (arg(0), arg(1), arg(2));
                    x.max(lo).min(hi)
                }
                Func::Abs => arg(0).abs(),
                Func::Floor => arg(0).floor(),
                Func::Ceil => arg(0).ceil(),
                Func::Round => arg(0).round(),
                Func::Exp => arg(0).exp(),
                Func::Frac => arg(0) - arg(0).floor(),
                Func::Log => arg(0).ln(),
                Func::Pow => arg(0).powf(arg(1)),
                Func::Sqrt => arg(0).sqrt(),
                Func::Signum => {
                    let x = arg(0);
                    if x > 0.0 {
                        1.0
                    } else if x < 0.0 {
                        -1.0
                    } else {
                        0.0
                    }
                }
                Func::Fmod => arg(0).rem_euclid(arg(1)),
                Func::Lerp => {
                    let (k, a, b) = (arg(0), arg(1), arg(2));
                    a + k * (b - a)
                }
                Func::Between => {
                    let x = arg(0);
                    flag(x >= arg(1) && x <= arg(2))
                }
                Func::Equals => flag((arg(0) - arg(1)).abs() <= arg(2)),
                Func::In => {
                    let x = arg(0);
                    flag(args.iter().skip(1).any(|a| ev(a) == x))
                }
                Func::Random => {
                    if args.is_empty() {
                        seeded(f64::from(std::process::id()) + vars.len() as f64)
                    } else {
                        seeded(arg(0))
                    }
                }
                Func::RandomB => flag(seeded(arg(0)) >= 0.5),
                Func::WrapDeg => wrap(arg(0), 180.0),
                Func::WrapRad => wrap(arg(0), std::f64::consts::PI),
                Func::DegDiff => wrap(arg(1) - arg(0), 180.0).abs(),
                Func::RadDiff => wrap(arg(1) - arg(0), std::f64::consts::PI).abs(),
                Func::Print => args.last().map_or(0.0, ev),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Model;
    impl Names for Model {
        fn part(&mut self, name: &str) -> Option<usize> {
            ["head", "body", "left_arm"].iter().position(|p| *p == name)
        }
        fn input(&self, name: &str) -> Option<usize> {
            ["limb_swing", "is_on_ground", "frame_time"]
                .iter()
                .position(|i| *i == name)
        }
    }

    fn run(lines: &[(&str, &str)], inputs: &[f64]) -> (Program, Vec<f64>, Vec<[f32; 9]>) {
        let mut p = Program::default();
        let errors = p.add(lines.iter().copied(), &mut Model);
        assert!(errors.is_empty(), "{errors:?}");
        let mut vars = Vec::new();
        let mut parts = vec![[0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 1.0, 1.0, 1.0]; 3];
        p.run(&mut vars, inputs, &mut parts);
        (p, vars, parts)
    }

    #[test]
    fn arithmetic_binds_as_the_games_does() {
        let (_, vars, _) = run(
            &[
                ("var.a", "1 + 2 * 3 - 4 / 2"),
                ("var.b", "-2 * -3 % 4"),
                ("var.c", "1 < 2 && 2 >= 2 || 0"),
                ("var.d", "!(1 == 1) + (3 != 2)"),
                ("var.e", "clamp(5, -1, 1) + abs(-0.5) + torad(180)"),
                ("var.f", "min(3, 1, 2) + max(-1, -5) + .5 + 1e1"),
            ],
            &[],
        );
        assert_eq!(vars[0], 5.0);
        assert_eq!(vars[1], 2.0);
        assert_eq!(vars[2], 1.0);
        assert_eq!(vars[3], 1.0);
        assert!((vars[4] - (1.5 + std::f64::consts::PI)).abs() < 1e-12);
        assert_eq!(vars[5], 1.0 - 1.0 + 0.5 + 10.0);
    }

    #[test]
    fn if_picks_its_first_true_branch_and_runs_only_that() {
        let (_, vars, _) = run(
            &[
                ("var.x", "if(0, 1 / 0, 0, 2, 3)"),
                ("var.y", "if(is_on_ground, 10, 20)"),
                ("var.z", "if(1 > 0, 7)"),
            ],
            &[0.0, 1.0, 0.0],
        );
        assert_eq!(vars, vec![3.0, 10.0, 7.0]);
    }

    #[test]
    fn variables_keep_their_value_and_parts_read_what_is_there() {
        let lines = [
            ("var.t", "var.t + frame_time"),
            ("head.rx", "head.rx + var.t"),
            ("body.ty", "head.rx * 10"),
            ("left_arm.rz", "unknown_value + 1"),
        ];
        let mut p = Program::default();
        assert!(p.add(lines.iter().copied(), &mut Model).is_empty());
        assert_eq!(p.unknown, ["unknown_value"]);
        let mut vars = Vec::new();
        for _ in 0..3 {
            let mut parts = vec![[0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 1.0, 1.0, 1.0]; 3];
            p.run(&mut vars, &[0.0, 0.0, 0.5], &mut parts);
            if vars[0] == 1.5 {
                assert!((parts[0][3] - 1.6).abs() < 1e-6);
                assert!((parts[1][1] - 16.0).abs() < 1e-5);
                assert_eq!(parts[2][5], 1.0);
            }
        }
        assert_eq!(vars[0], 1.5);
    }

    #[test]
    fn a_line_that_does_not_parse_is_left_out_with_why() {
        let mut p = Program::default();
        let errors = p.add(
            [
                ("var.a", "1 +"),
                ("var.b", "nofunc(1)"),
                ("var.c", "(1"),
                ("var.d", "2"),
            ],
            &mut Model,
        );
        assert_eq!(errors.len(), 3);
        assert_eq!(p.lines.len(), 1);
    }

    #[test]
    fn random_is_the_same_for_a_seed() {
        let (_, vars, _) = run(
            &[
                ("var.a", "random(5)"),
                ("var.b", "random(5)"),
                ("var.c", "randomb(3) + randomb(3) * 0"),
                ("var.d", "wrapdeg(190) + wraprad(0)"),
            ],
            &[],
        );
        assert_eq!(vars[0], vars[1]);
        assert!((0.0..1.0).contains(&vars[0]));
        assert!(vars[2] == 0.0 || vars[2] == 1.0);
        assert_eq!(vars[3], -170.0);
    }
}
