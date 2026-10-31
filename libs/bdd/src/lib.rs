//! Given, when, then tests with no dependencies.
//!
//! ```ignore
//! #[spec]
//! fn walking() {
//!     given("two players", || {
//!         let mut players = [0.0, 0.0];
//!
//!         when("player 1 walks", || {
//!             players[0] += 1.0;
//!
//!             test("player 1 moves", || {
//!                 assert_eq!(players[0], 1.0);
//!             });
//!
//!             test("player 2 stays still", || {
//!                 assert_eq!(players[1], 0.0);
//!             });
//!         });
//!     });
//! }
//! ```
//!
//! The body is valid Rust syntax, so `rustfmt` formats it, but `given`, `when`,
//! and `test` aren't real functions. The macro turns each `test` into a normal
//! `#[test]` function, with the setup above it copied in, so every test runs its
//! own fresh setup. The example becomes
//! `walking::given_two_players::when_player_1_walks::player_1_moves`.

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use std::collections::BTreeMap;

const STEPS: [&str; 3] = ["given", "when", "test"];

const RUST_KEYWORDS: [&str; 37] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
    "where", "while",
];

/// Turns a function of `given`, `when`, and `test` steps into `#[test]` functions.
#[proc_macro_attribute]
pub fn spec(_args: TokenStream, item: TokenStream) -> TokenStream {
    match expand(item) {
        Ok(tokens) => tokens,
        Err(error) => error.into_tokens(),
    }
}

struct SpecError {
    message: String,
    span: Span,
}

impl SpecError {
    fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }

    fn into_tokens(self) -> TokenStream {
        let mut bang = Punct::new('!', Spacing::Alone);
        bang.set_span(self.span);
        let mut message = Literal::string(&self.message);
        message.set_span(self.span);
        let mut arguments = Group::new(
            Delimiter::Parenthesis,
            TokenStream::from(TokenTree::Literal(message)),
        );
        arguments.set_span(self.span);
        TokenStream::from_iter([
            TokenTree::Ident(Ident::new("compile_error", self.span)),
            TokenTree::Punct(bang),
            TokenTree::Group(arguments),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ])
    }
}

type Parsed<T> = Result<T, SpecError>;

struct Step {
    keyword: String,
    description: String,
    span: Span,
    body: TokenStream,
}

struct Block {
    setup: TokenStream,
    steps: Vec<Step>,
}

/// Generated Rust modules. Scenarios that share a description share a module.
#[derive(Default)]
struct Scope {
    children: BTreeMap<String, Scope>,
    tests: Vec<(String, TokenStream)>,
}

fn expand(item: TokenStream) -> Parsed<TokenStream> {
    let tokens: Vec<TokenTree> = item.into_iter().collect();
    let keyword = tokens.iter().position(is_fn_keyword);
    let name = keyword.and_then(|index| match tokens.get(index + 1) {
        Some(TokenTree::Ident(name)) => Some(name.to_string()),
        _ => None,
    });
    let body = match tokens.last() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
            Some(group.stream())
        }
        _ => None,
    };
    let (Some(name), Some(body)) = (name, body) else {
        return Err(SpecError::new(
            "#[spec] must be placed on a function",
            Span::call_site(),
        ));
    };

    let root = parse_spec(body)?;
    let mut output: TokenStream = "#[cfg(test)]".parse().expect("valid attribute");
    output.extend(scope_tokens(&name, &root));
    Ok(output)
}

fn is_fn_keyword(token: &TokenTree) -> bool {
    matches!(token, TokenTree::Ident(ident) if ident.to_string() == "fn")
}

fn parse_spec(body: TokenStream) -> Parsed<Scope> {
    let top = parse_block(body)?;
    if let Some(token) = top.setup.into_iter().next() {
        return Err(SpecError::new(
            "#[spec] functions may only contain `given` steps",
            token.span(),
        ));
    }
    if top.steps.is_empty() {
        return Err(SpecError::new(
            "#[spec] needs a `given` step",
            Span::call_site(),
        ));
    }

    let mut root = Scope::default();
    for given in top.steps {
        require_step(&given, "given", "the top of a #[spec] function")?;
        let given_block = parse_block(given.body)?;
        require_children(&given_block, given.span, "`given` needs a `when` step")?;
        let given_name = format!("given_{}", slug(&given.description, given.span)?);

        for when in given_block.steps {
            require_step(&when, "when", "a `given` step")?;
            let when_block = parse_block(when.body)?;
            require_children(&when_block, when.span, "`when` needs a `test` step")?;
            let when_name = format!("when_{}", slug(&when.description, when.span)?);

            for test in when_block.steps {
                require_step(&test, "test", "a `when` step")?;
                let test_block = parse_block(test.body.clone())?;
                if let Some(nested) = test_block.steps.first() {
                    return Err(SpecError::new(
                        "`test` can't contain another step",
                        nested.span,
                    ));
                }

                let mut body = given_block.setup.clone();
                body.extend(when_block.setup.clone());
                body.extend(test.body);
                root.children
                    .entry(given_name.clone())
                    .or_default()
                    .children
                    .entry(when_name.clone())
                    .or_default()
                    .tests
                    .push((test_name(&test.description, test.span)?, body));
            }
        }
    }
    Ok(root)
}

fn require_step(step: &Step, keyword: &str, place: &str) -> Parsed<()> {
    if step.keyword == keyword {
        Ok(())
    } else {
        Err(SpecError::new(
            format!("`{}` can't be used directly in {place}", step.keyword),
            step.span,
        ))
    }
}

fn require_children(block: &Block, span: Span, message: &str) -> Parsed<()> {
    if block.steps.is_empty() {
        Err(SpecError::new(message, span))
    } else {
        Ok(())
    }
}

/// Splits a step's body into its setup code and its nested steps.
fn parse_block(stream: TokenStream) -> Parsed<Block> {
    let mut tokens = stream.into_iter().peekable();
    let mut setup = TokenStream::new();
    let mut steps = Vec::new();
    let mut at_statement_start = true;

    while let Some(token) = tokens.next() {
        if let (true, TokenTree::Ident(ident)) = (at_statement_start, &token) {
            let keyword = ident.to_string();
            let is_call = matches!(
                tokens.peek(),
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis
            );
            if is_call && STEPS.contains(&keyword.as_str()) {
                let Some(TokenTree::Group(arguments)) = tokens.next() else {
                    unreachable!("the next token was checked above");
                };
                steps.push(parse_step(keyword, ident.span(), arguments.stream())?);
                if matches!(tokens.peek(), Some(TokenTree::Punct(p)) if p.as_char() == ';') {
                    tokens.next();
                }
                continue;
            }
        }

        if !steps.is_empty() {
            return Err(SpecError::new(
                "setup must come before nested steps",
                token.span(),
            ));
        }
        at_statement_start = match &token {
            TokenTree::Punct(punct) => punct.as_char() == ';',
            TokenTree::Group(group) => group.delimiter() == Delimiter::Brace,
            _ => false,
        };
        setup.extend([token]);
    }
    Ok(Block { setup, steps })
}

/// Reads `"description", || { body }`.
fn parse_step(keyword: String, span: Span, arguments: TokenStream) -> Parsed<Step> {
    let tokens: Vec<TokenTree> = arguments.into_iter().collect();
    let usage = || {
        SpecError::new(
            format!("expected {keyword}(\"description\", || {{ ... }})"),
            span,
        )
    };

    let [
        TokenTree::Literal(description),
        TokenTree::Punct(comma),
        TokenTree::Punct(open),
        TokenTree::Punct(close),
        TokenTree::Group(body),
        ..,
    ] = tokens.as_slice()
    else {
        return Err(usage());
    };
    let is_closure = comma.as_char() == ',' && open.as_char() == '|' && close.as_char() == '|';
    if !is_closure || body.delimiter() != Delimiter::Brace {
        return Err(usage());
    }

    let text = description.to_string();
    let Some(description) = text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
    else {
        return Err(SpecError::new("the description must be a string", span));
    };

    Ok(Step {
        keyword,
        description: description.to_owned(),
        span,
        body: body.stream(),
    })
}

/// Turns a description into `lower_snake_case` so it can name Rust items.
fn slug(description: &str, span: Span) -> Parsed<String> {
    let lowered = description.to_lowercase();
    let words: Vec<&str> = lowered
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    if words.is_empty() {
        return Err(SpecError::new(
            "the description needs at least one letter or number",
            span,
        ));
    }
    Ok(words.join("_"))
}

fn test_name(description: &str, span: Span) -> Parsed<String> {
    let name = slug(description, span)?;
    if RUST_KEYWORDS.contains(&name.as_str()) {
        Ok(format!("{name}_"))
    } else if name.starts_with(|first: char| first.is_ascii_digit()) {
        Ok(format!("_{name}"))
    } else {
        Ok(name)
    }
}

fn scope_tokens(name: &str, scope: &Scope) -> TokenStream {
    let mut contents: TokenStream = "#[allow(unused_imports)] use super::*;"
        .parse()
        .expect("valid import");
    for (child_name, child) in &scope.children {
        contents.extend(scope_tokens(child_name, child));
    }
    for (test, body) in &scope.tests {
        contents.extend(test_tokens(test, body.clone()));
    }

    let mut output: TokenStream = format!("mod {name}").parse().expect("valid module name");
    output.extend([TokenTree::Group(Group::new(Delimiter::Brace, contents))]);
    output
}

fn test_tokens(name: &str, body: TokenStream) -> TokenStream {
    // Not every test in a scenario uses all of its shared setup.
    let mut output: TokenStream =
        format!("#[test] #[allow(unused_mut, unused_variables)] fn {name}()")
            .parse()
            .expect("valid test function");
    output.extend([TokenTree::Group(Group::new(Delimiter::Brace, body))]);
    output
}
