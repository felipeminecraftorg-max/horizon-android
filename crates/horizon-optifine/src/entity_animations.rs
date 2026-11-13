//! Bounded CEM/EMF expression compiler. Compiles once at pack load, evaluates per frame.
//! Partial OptiFine expression support; unsupported symbols fail closed.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    TooLong, InvalidToken(usize), UnexpectedToken, UnexpectedEnd,
    TooDeep, UnknownVariable(String), UnknownFunction(String),
    Arity, NonFinite, DivisionByZero,
}
#[derive(Clone, Debug, PartialEq)]
enum Token {
    Num(f64), Ident(String), Op(&'static str), LParen, RParen, Comma,
}
#[derive(Clone, Debug)]
enum Node {
    Number(f64), Variable(String),
    Unary(&'static str, Box<Node>),
    Binary(&'static str, Box<Node>, Box<Node>),
    Call(String, Vec<Node>),
}
#[derive(Clone, Debug)]
pub struct Compiled { ast: Node }
fn tokenize(text: &str) -> Result<Vec<Token>, Error> {
    if text.len() > 4096 { return Err(Error::TooLong); }
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() { i += 1; continue; }
        let start = i;
        let t = match bytes[i] {
            b'(' => { i += 1; Token::LParen },
            b')' => { i += 1; Token::RParen },
            b',' => { i += 1; Token::Comma },
            b'0'..=b'9' | b'.' => {
                i += 1;
                while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') { i += 1; }
                if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
                    i += 1;
                    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') { i += 1; }
                    let e = i;
                    while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
                    if i == e { return Err(Error::InvalidToken(start)); }
                }
                let n: f64 = text[start..i].parse().map_err(|_| Error::InvalidToken(start))?;
                if !n.is_finite() { return Err(Error::NonFinite); }
                Token::Num(n)
            },
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                i += 1;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.') { i += 1; }
                Token::Ident(text[start..i].into())
            },
            _ => {
                let rest = &text[i..];
                let op = if rest.starts_with("&&") { "&&" }
                    else if rest.starts_with("||") { "||" }
                    else if rest.starts_with(">=") { ">=" }
                    else if rest.starts_with("<=") { "<=" }
                    else if rest.starts_with("==") { "==" }
                    else if rest.starts_with("!=") { "!=" }
                    else { match bytes[i] {
                        b'+' => "+", b'-' => "-", b'*' => "*", b'/' => "/",
                        b'%' => "%", b'^' => "^", b'>' => ">", b'<' => "<", b'!' => "!",
                        _ => return Err(Error::InvalidToken(i)),
                    }};
                i += op.len();
                Token::Op(op)
            },
        };
        out.push(t);
        if out.len() > 1024 { return Err(Error::TooLong); }
    }
    Ok(out)
}
fn binding(op: &str) -> Option<(u8,u8)> {
    Some(match op {
        "||" => (1,2), "&&" => (3,4),
        "==" | "!=" => (5,6),
        "<" | "<=" | ">" | ">=" => (7,8),
        "+" | "-" => (9,10),
        "*" | "/" | "%" => (11,12),
        "^" => (15,14), // right associative
        _ => return None,
    })
}
struct Parser { tokens: Vec<Token>, pos: usize }
impl Parser {
    fn next(&mut self) -> Option<Token> { let t = self.tokens.get(self.pos).cloned(); self.pos += usize::from(t.is_some()); t }
    fn peek(&self) -> Option<&Token> { self.tokens.get(self.pos) }
    fn expr(&mut self, min: u8, depth: usize) -> Result<Node, Error> {
        if depth > 32 { return Err(Error::TooDeep); }
        let mut left = match self.next().ok_or(Error::UnexpectedEnd)? {
            Token::Num(n) => Node::Number(n),
            Token::Ident(name) => {
                if self.peek() == Some(&Token::LParen) {
                    self.next();
                    let mut args = Vec::new();
                    if self.peek() != Some(&Token::RParen) {
                        loop {
                            args.push(self.expr(0, depth+1)?);
                            if args.len() > 16 { return Err(Error::Arity); }
                            if self.peek() != Some(&Token::Comma) { break; }
                            self.next();
                        }
                    }
                    if self.next() != Some(Token::RParen) { return Err(Error::UnexpectedToken); }
                    Node::Call(name, args)
                } else { Node::Variable(name) }
            },
            Token::Op("-") => Node::Unary("-", Box::new(self.expr(13, depth+1)?)),
            Token::Op("+") => self.expr(13, depth+1)?,
            Token::Op("!") => Node::Unary("!", Box::new(self.expr(13, depth+1)?)),
            Token::LParen => {
                let n = self.expr(0, depth+1)?;
                if self.next() != Some(Token::RParen) { return Err(Error::UnexpectedToken); }
                n
            },
            _ => return Err(Error::UnexpectedToken),
        };
        while let Some(Token::Op(op)) = self.peek() {
            let Some((left_bp,right_bp)) = binding(op) else { break };
            if left_bp < min { break; }
            let op = *op;
            self.next();
            let right = self.expr(right_bp, depth+1)?;
            left = Node::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
}

fn validate(node: &Node) -> Result<(), Error> {
    match node {
        Node::Number(_) | Node::Variable(_) => Ok(()),
        Node::Unary(_, child) => validate(child),
        Node::Binary(_, left, right) => { validate(left)?; validate(right) },
        Node::Call(name, args) => {
            let count = args.len();
            let valid = match name.as_str() {
                "sin" | "cos" | "tan" | "asin" | "acos" | "atan"
                | "abs" | "sqrt" | "floor" | "ceil" | "round" | "frac" => count == 1,
                "atan2" | "pow" | "mod" => count == 2,
                "clamp" | "lerp" => count == 3,
                "min" | "max" => count >= 1,
                "if" => count >= 3 && count % 2 == 1,
                _ => return Err(Error::UnknownFunction(name.clone())),
            };
            if !valid { return Err(Error::Arity); }
            for arg in args { validate(arg)?; }
            Ok(())
        }
    }
}

impl Compiled {
    pub fn parse(text: &str) -> Result<Self, Error> {
        let mut p = Parser { tokens: tokenize(text)?, pos: 0 };
        let ast = p.expr(0, 0)?;
        if p.pos != p.tokens.len() { return Err(Error::UnexpectedToken); }
        validate(&ast)?;
        Ok(Self { ast })
    }
    /// Variables come from the renderer's current entity state; never from filesystem or scripts.
    pub fn eval<F: Fn(&str) -> Option<f64>>(&self, variable: &F) -> Result<f64, Error> {
        let value = evaluate(&self.ast, variable, 0)?;
        if value.is_finite() { Ok(value) } else { Err(Error::NonFinite) }
    }
}
fn truthy(v: f64) -> bool { v != 0.0 }
fn evaluate<F: Fn(&str)->Option<f64>>(node: &Node, vars: &F, depth: usize) -> Result<f64, Error> {
    if depth > 64 { return Err(Error::TooDeep); }
    let next = depth + 1;
    let eval = |n: &Node| evaluate(n, vars, next);
    let value = match node {
        Node::Number(n) => *n,
        Node::Variable(name) => match name.as_str() {
            "pi" => std::f64::consts::PI,
            "true" => 1.0, "false" => 0.0,
            _ => vars(name).ok_or_else(|| Error::UnknownVariable(name.clone()))?,
        },
        Node::Unary("-", x) => -eval(x)?,
        Node::Unary("!", x) => f64::from(!truthy(eval(x)?)),
        Node::Unary(_, _) => return Err(Error::UnexpectedToken),
        Node::Binary("&&", a, b) => f64::from(truthy(eval(a)?) && truthy(eval(b)?)),
        Node::Binary("||", a, b) => f64::from(truthy(eval(a)?) || truthy(eval(b)?)),
        Node::Binary(op, a, b) => {
            let a = eval(a)?; let b = eval(b)?;
            match *op {
                "+" => a+b, "-" => a-b, "*" => a*b,
                "/" => { if b == 0.0 { return Err(Error::DivisionByZero); } a/b },
                "%" => { if b == 0.0 { return Err(Error::DivisionByZero); } a%b },
                "^" => a.powf(b),
                "==" => f64::from(a == b), "!=" => f64::from(a != b),
                ">" => f64::from(a > b), ">=" => f64::from(a >= b),
                "<" => f64::from(a < b), "<=" => f64::from(a <= b),
                _ => return Err(Error::UnexpectedToken),
            }
        },
        Node::Call(name, args) if name == "if" => {
            if args.len() < 3 || args.len() % 2 == 0 { return Err(Error::Arity); }
            let mut i = 0;
            while i + 1 < args.len()-1 {
                if truthy(eval(&args[i])?) { return eval(&args[i+1]); }
                i += 2;
            }
            return eval(args.last().ok_or(Error::Arity)?);
        },
        Node::Call(name, args) => {
            let vals = args.iter().map(eval).collect::<Result<Vec<_>,_>>()?;
            match (name.as_str(), vals.as_slice()) {
                ("sin", [x]) => x.sin(), ("cos", [x]) => x.cos(), ("tan", [x]) => x.tan(),
                ("asin", [x]) => x.asin(), ("acos", [x]) => x.acos(),
                ("atan", [x]) => x.atan(), ("atan2", [a,b]) => a.atan2(*b),
                ("abs", [x]) => x.abs(), ("sqrt", [x]) => x.sqrt(),
                ("floor", [x]) => x.floor(), ("ceil", [x]) => x.ceil(),
                ("round", [x]) => x.round(),
                ("min", xs) if !xs.is_empty() => xs.iter().copied().reduce(f64::min).unwrap_or(0.0),
                ("max", xs) if !xs.is_empty() => xs.iter().copied().reduce(f64::max).unwrap_or(0.0),
                ("pow", [x,y]) => x.powf(*y),
                ("clamp", [x,a,b]) if a <= b => x.clamp(*a,*b),
                ("clamp", [_,_,_]) => return Err(Error::UnexpectedToken),
                ("lerp", [t,a,b]) => a + t*(b-a),
                ("mod", [a,b]) if *b != 0.0 => a.rem_euclid(*b),
                ("mod", [_,_]) => return Err(Error::DivisionByZero),
                ("frac", [x]) => x.fract(),
                (name, _) => return Err(Error::UnknownFunction(name.into())),
            }
        }
    };
    if value.is_finite() { Ok(value) } else { Err(Error::NonFinite) }
}

/// Animation source from parsed CEM parts, compiled once. Each target is a
/// named model property; the renderer must resolve that property and apply it.
#[derive(Clone, Debug)]
pub struct Animation {
    pub target: String,
    expression: Compiled,
}
impl Animation {
    pub fn compile(target: &str, expression: &str) -> Result<Self, Error> {
        if target.len() > 128 || !target.contains('.') { return Err(Error::UnexpectedToken); }
        Ok(Self { target: target.to_owned(), expression: Compiled::parse(expression)? })
    }
    pub fn sample<F: Fn(&str)->Option<f64>>(&self, vars: &F) -> Result<f32, Error> {
        let value = self.expression.eval(vars)?;
        if value.abs() > f32::MAX as f64 { return Err(Error::NonFinite); }
        Ok(value as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn precedence_and_constants() {
        assert_eq!(Compiled::parse("1+2*3").unwrap().eval(&|_| None), Ok(7.0));
        assert_eq!(Compiled::parse("2^3^2").unwrap().eval(&|_| None), Ok(512.0));
    }
    #[test] fn lazy_if_and_bool() {
        let e = Compiled::parse("if(is_baby && limb_swing > 0, 0.5, 1/0)").unwrap();
        assert_eq!(e.eval(&|name| match name {"is_baby"|"limb_swing"=>Some(1.0),_=>None}), Ok(0.5));
    }
    #[test] fn sampled_pose_variable() {
        let a = Animation::compile("head.rx", "sin(limb_swing)*0.5").unwrap();
        assert_eq!(a.sample(&|name| (name=="limb_swing").then_some(0.0)), Ok(0.0));
    }
    #[test] fn invalid_expression_fails() {
        assert!(Compiled::parse("sin(").is_err());
        assert!(Compiled::parse("1 + self.execute()").is_err());
        assert!(matches!(Compiled::parse("clamp(1,2)"), Err(Error::Arity)));
    }
    #[test] fn mathematical_domain_fails() {
        assert_eq!(Compiled::parse("sqrt(-1)").unwrap().eval(&|_|None), Err(Error::NonFinite));
    }
}
