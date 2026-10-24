#![allow(non_upper_case_globals)]

use crate::{
    Result,
    buffer::Buffer,
    node::*,
    token::{*, consts::*},
    tokenizer::TokenMeta,
    types::Type,
};

pub struct Parser {
    pub tokens: Buffer<Token>,
    metas: Vec<TokenMeta>,
}

impl Parser {
    pub fn new(tokens: Buffer<Token>, metas: Vec<TokenMeta>) -> Self {
        Self {
            tokens,
            metas,
        }
    }

    fn get_meta(&self) -> NodeMeta {
        let TokenMeta { row, col } = self.metas[self.tokens.i];
        NodeMeta {
            location: Some((row, col)),
            ty: Type::Unknown,
        }
    }

    fn read_value(&mut self) -> Result<Node> {
        let meta = self.get_meta();

        let un_op = if let Token::Operator(op) = self.tokens.peek()?.clone() {
            self.tokens.step();
            Some(op)
        } else {
            None
        };

        let mut value = {
            let meta = meta.clone();
            match self.tokens.next()? {
                Token::String(string) => Node::String(string.clone().into(), meta),
                Token::Integer(int) => Node::Integer(*int, meta),
                Token::Float(float) => Node::Float(*float, meta),
                Token::Boolean(boolean) => Node::Boolean(*boolean, meta),
                Token::Identifier(ident)
                    => Node::Identifier(ident.clone().into(), meta),
                &ParenOpen => {
                    let expr = self.read_expression()?;
                    self.tokens.expect(&ParenClose)?;
                    expr
                }
                &BracketOpen =>
                    Node::List(self.read_args(Separator::BracketClose)?, meta),
                &BraceOpen => {
                    self.tokens.back();
                    self.read_block(true)?
                },
                &True => Node::Boolean(true, meta),
                &False => Node::Boolean(false, meta),
                token => { return Err(format!("expected value (found {token:?})").into()) }
            }
        };

        while let Ok(token) = self.tokens.peek() {
            match token {
                &ParenOpen => {
                    self.tokens.step();
                    value = Node::ParenArgs(
                        Box::new(value),
                        self.read_args(Separator::ParenClose)?,
                        meta.clone(),
                    );
                }
                _ => { break }
            }
        }

        let value = if let Some(op) = un_op {
            Node::UnaryOp(
                Box::new(UnaryOp {
                    op: op.clone(),
                    value,
                }),
                meta.clone(),
            )
        } else {
            value
        };

        Ok(value)
    }

    fn read_args(&mut self, terminator: Separator) -> Result<Vec<Node>> {
        let mut args = Vec::new();

        loop {
            let peek = self.tokens.peek()?;
            if let Token::Separator(op) = peek {
                if *op == terminator {
                    self.tokens.step();
                    break
                }
            }
            args.push(self.read_expression()?);
            let next = self.tokens.next()?;
            match next {
                &Comma => {
                    continue
                }
                Token::Separator(op) => {
                    if *op == terminator {
                        break
                    }
                }
                _ => {}
            }
            return Err(format!("unexpected token: {next:?}").into())
        }

        Ok(args)
    }

    fn read_expression(&mut self) -> Result<Node> {
        let mut values: Vec<Node> = Vec::new();
        let mut operators: Vec<Operator> = Vec::new();

        loop {
            values.push(self.read_value()?);

            if let Ok(Token::Operator(op)) = self.tokens.peek().cloned() {
                if BINARY_OPERATORS.contains(&op) {
                    self.tokens.step();
                    operators.push(op);
                } else {
                    break
                }
            } else {
                break
            }
        }

        while 0 < operators.len() {
            'levels: for level in OPERATOR_ORDER {
                for target_op in level.iter() {
                    for (i, op) in operators.clone().iter().enumerate() {
                        if op == target_op {
                            let [a, b]: [Node; 2] = values.splice(i..=i+1, [])
                                .collect::<Vec<_>>().try_into().unwrap();
                            let _ = operators.remove(i);

                            values.insert(i, Node::BinaryOp(
                                Box::new(BinaryOp {
                                    op: op.clone(),
                                    a,
                                    b,
                                }),
                                NodeMeta::empty(),
                            ));

                            break 'levels
                        }
                    }
                }
            }
        }

        assert!(values.len() == 1);
        Ok(values.remove(0))
    }

    fn read_if(&mut self, meta: NodeMeta) -> Result<Node> {
        let mut statements = vec![(
            Box::new(self.read_expression()?),
            Box::new(self.read_block(true)?),
        )];

        let mut end = None;

        loop {
            match self.tokens.peek() {
                Ok(&Elif) => {
                    self.tokens.step();
                    statements.push((
                        Box::new(self.read_expression()?),
                        Box::new(self.read_block(true)?),
                    ));
                }
                Ok(&Else) => {
                    self.tokens.step();
                    end = Some(Box::new(self.read_block(true)?));
                    break
                }
                _ => {
                    break
                }
            }
        }

        Ok(Node::If(statements, end, meta))
    }

    fn read_ident_as_string(&mut self) -> Result<String> {
        if let Token::Identifier(ident) = self.tokens.next()?.clone() {
            Ok(ident)
        } else {
            Err("expected identifier".into())
        }
    }

    fn read_for(&mut self, meta: NodeMeta) -> Result<Node> {
        let ident = self.read_ident_as_string()?;
        self.tokens.expect(&In)?;
        let expr = self.read_expression()?;
        let block = self.read_block(true)?;
        Ok(Node::For(
            ident.into(),
            Box::new(expr),
            Box::new(block),
            meta,
        ))
    }

    fn read_while(&mut self, meta: NodeMeta) -> Result<Node> {
        let expr = self.read_expression()?;
        let block = self.read_block(true)?;
        Ok(Node::While(
            Box::new(expr),
            Box::new(block),
            meta,
        ))
    }

    fn read_let(&mut self, meta: NodeMeta) -> Result<Node> {
        let ident = self.read_ident_as_string()?;
        self.tokens.expect(&Assign)?;
        let expr = self.read_expression()?;
        self.tokens.expect(&Semicolon)?;
        Ok(Node::DefineVariable(
            ident.to_string().into(),
            Box::new(expr),
            meta,
        ))
    }

    fn read_goto(&mut self, meta: NodeMeta) -> Result<Node> {
        let ident = self.read_ident_as_string()?;
        self.tokens.expect(&Semicolon)?;
        Ok(Node::Goto(ident.into(), meta))
    }

    fn read_func(&mut self, meta: NodeMeta) -> Result<Node> {
        let ident = self.read_ident_as_string()?;
        self.tokens.expect(&ParenOpen)?;
        let mut args = Vec::new();
        for arg in self.read_args(Separator::ParenClose)? {
            if let Node::Identifier(ident, _) = arg {
                args.push(ident);
            } else {
                return Err("expected identifier".into())
            }
        }
        let block = self.read_block(true)?;
        Ok(Node::DefineFunction(
            ident.to_string().into(),
            args,
            Box::new(block),
            meta,
        ))
    }

    fn read_block(&mut self, inner: bool) -> Result<Node> {
        let meta = self.get_meta();
        let mut nodes = Vec::new();
        let mut return_last = false;

        if inner {
            self.tokens.expect(&BraceOpen)?;
        }

        loop {
            match self.tokens.peek().cloned() {
                Ok(BraceClose) => {
                    if inner {
                        self.tokens.step();
                        break
                    } else {
                        return Err("unexpected '}'".into())
                    }
                }
                Ok(BraceOpen) => {
                    nodes.push(self.read_block(true)?);
                }
                Ok(Token::Keyword(kw)) => {
                    use Keyword::*;
                    let meta = self.get_meta();
                    self.tokens.step();
                    nodes.push(match kw {
                        If => self.read_if(meta)?,
                        For => self.read_for(meta)?,
                        While => self.read_while(meta)?,
                        Func => self.read_func(meta)?,
                        Let => self.read_let(meta)?,
                        Goto => self.read_goto(meta)?,
                        True | False | Elif | Else | In | KwAnd | KwOr | KwNot
                            => { return Err("unexpected keyword".into()) }
                    });
                }
                Ok(Colon) => {
                    let meta = self.get_meta();
                    self.tokens.step();
                    nodes.push(Node::Label(
                        self.read_ident_as_string()?.into(),
                        meta,
                    ));
                    self.tokens.expect(&Semicolon)?;
                }
                Ok(_) => {
                    nodes.push(self.read_expression()?);
                    match self.tokens.peek()? {
                        &Semicolon => self.tokens.step(),
                        &BraceClose => {
                            return_last = true;
                        }
                        token => { return Err(format!(
                            "expected ';' or '}}' (found {token:?})").into())
                        }
                    }
                }
                Err(_) => {
                    break
                }
            }
        }

        Ok(Node::Block(
            Block {
                nodes,
                return_last,
            },
            meta,
        ))
    }

    pub fn parse(&mut self) -> Result<Node> {
        self.read_block(false)
    }
}
