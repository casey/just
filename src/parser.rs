#[derive(Debug, Eq, PartialEq)]
pub enum Token {
	Text(String),
	Env(String),
	Space,
	Pipe,
	RedirectIn,
	RedirectOut
}

impl Token {
	fn is_empty(&self) -> bool {
		match *self {
			Token::Text(ref buffer) => buffer.is_empty(),
			Token::Env(ref buffer) => buffer.is_empty(),
			_ => false,
		}
	}
}

pub fn parse(input: &str) -> Vec<Token> {
	let mut tokens = Vec::new();
	let mut buffer = Token::Text(String::new());
	let mut escape = false;
	let mut quote  = None;

	'chars:
	for c in input.chars() {
		loop {
			let mut repeat = false;
			let mut flush  = false;
			let mut space  = false;
			let mut newbuffer = None;
			match buffer {
				Token::Text(ref mut buffer) => {
					if escape {
						buffer.push(c);
						escape = false;
						continue 'chars;
					}
					match c {
						'\\' => escape = true,
						'\"' | '\'' if quote.is_none()  => quote = Some(c),
						'\"' | '\'' if quote == Some(c) => quote = None,
						' ' if quote.is_none() => {
							flush = true;
							space = true;
							newbuffer = Some(Token::Text(String::new()));
						},
						'$' => {
							flush = true;
							newbuffer = Some(Token::Env(String::new()));
						},
						'|' if quote.is_none() && buffer.is_empty() => {
							flush = true;
							space = true;
							newbuffer = Some(Token::Pipe)
						},
						'<' if quote.is_none() && buffer.is_empty() => {
							flush = true;
							space = true;
							newbuffer = Some(Token::RedirectIn)
						},
						'>' if quote.is_none() && buffer.is_empty() => {
							flush = true;
							space = true;
							newbuffer = Some(Token::RedirectOut)
						},
						_ => buffer.push(c),
					}
				},
				Token::Env(ref mut buffer) => {
					if buffer.is_empty() && c == '{' {
						quote = Some('}');
					} else if quote == Some(c) {
						flush = true;
						newbuffer = Some(Token::Text(String::new()));
					} else {
						let code = c as u32;
						if quote.is_none() &&
							(code < 'a' as u32 || code > 'z' as u32) &&
							(code < 'A' as u32 || code > 'Z' as u32) &&
							(code < '0' as u32 || code > '9' as u32) {

							newbuffer = Some(Token::Text(if buffer.is_empty() {
								let mut newbuffer = String::with_capacity(1);
								newbuffer.push('$');
								newbuffer
							} else {
								flush = true;
								String::new()
							}));
							repeat = true;
						} else {
							buffer.push(c);
						}
					}
				},
				Token::Pipe | Token::RedirectIn | Token::RedirectOut => {
					flush = true;
					space = true;
					newbuffer = Some(Token::Text(String::new()))
				},
				_ => unreachable!(),
			}
			if let Some(newbuffer) = newbuffer {
				if flush && !buffer.is_empty() {
					tokens.push(buffer);

					if space {
						tokens.push(Token::Space);
					}
				}
				buffer = newbuffer;
			}
			if !repeat {
				break;
			}
		}
	}
	tokens.push(buffer);

	tokens
}

#[cfg(test)]
#[test]
fn test() {
	macro_rules! text {
		($text:expr) => {
			Token::Text($text.to_string())
		}
	}
	macro_rules! env {
		($text:expr) => {
			Token::Env($text.to_string())
		}
	}

	assert_eq!(
		parse(r#"echo "\"HELLO\", WORLD"lol $TEST. ${LOL!}lol"#),
		vec![text!("echo"), Token::Space, text!("\"HELLO\", WORLDlol"), Token::Space, env!("TEST"),
			text!("."), Token::Space, env!("LOL!"), text!("lol")]
	);
	assert_eq!(
		parse(r#"cat < in > out"#),
		vec![text!("cat"), Token::Space, Token::RedirectIn, Token::Space, text!("in"), Token::Space,
			Token::RedirectOut, Token::Space, text!("out")]
	);
	assert_eq!(
		parse(r#"echo '<'"#),
		vec![text!("echo"), Token::Space, text!("<")]
	);
}
