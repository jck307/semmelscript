use crate::{
    Result,
    buffer::Buffer,
    node::*,
    token::{*, Keyword::*},
};

pub struct Parser {
    pub tokens: Buffer<Token>,
}

impl Parser {
    pub fn new(tokens: Buffer<Token>) -> Self {
        Self {
            tokens,
        }
    }

    fn read_value(&mut self) -> Result<Node> {
        let mut value = match self.tokens.next()? {
            Token::String(string) => Node::String(string.clone().into()),
            Token::Integer(int) => Node::Integer(*int),
            Token::Boolean(boolean) => Node::Boolean(*boolean),
            Token::Identifier(ident) => Node::Identifier(ident.clone().into()),
            token => { return Err(format!("expected value (found {token:?})").into()) }
        };

        while let Ok(token) = self.tokens.peek() {
            match token {
                Token::Operator(Operator::ParenOpen) => {
                    self.tokens.step();
                    value = Node::ParenArgs(
                        Box::new(value),
                        self.read_args(Operator::ParenClose)?
                    );
                }
                _ => { break }
            }
        }

        Ok(value)
    }

    fn read_args(&mut self, terminator: Operator) -> Result<Vec<Node>> {
        let mut args = Vec::new();

        loop {
            let peek = self.tokens.peek()?;
            if let Token::Operator(op) = peek {
                if *op == terminator {
                    self.tokens.step();
                    break
                }
            }
            args.push(self.read_expression()?);
            let next = self.tokens.next()?;
            match next {
                Token::Operator(Operator::Comma) => {
                    continue
                }
                Token::Operator(op) => {
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

                            values.insert(i, Node::BinaryOp(Box::new(BinaryOp {
                                op: op.clone(),
                                a,
                                b,
                            })));

                            break 'levels
                        }
                    }
                }
            }
        }

        assert!(values.len() == 1);
        Ok(values[0].clone())
    }

    fn read_if(&mut self) -> Result<Node> {
        let condition = self.read_expression()?;
        let block = self.read_block(true)?;

        let ext: Option<Box<Node>> = if let Ok(Token::Keyword(kw)) = self.tokens.peek() {
            match kw {
                Keyword::Else => Some(Box::new(self.read_block(true)?)),
                Keyword::Elif => Some(Box::new(self.read_if()?)),
                _ => None
            }
        } else {
            None
        };

        Ok(Node::Statement(Statement::If(Box::new(condition), Box::new(block), ext)))
    }

    fn read_ident_as_string(&mut self) -> Result<String> {
        if let Token::Identifier(ident) = self.tokens.next()?.clone() {
            Ok(ident)
        } else {
            Err("expected identifier".into())
        }
    }

    fn read_for(&mut self) -> Result<Node> {
        let ident = self.read_ident_as_string()?;
        self.tokens.expect(&Token::Keyword(Keyword::In))?;
        let expr = self.read_expression()?;
        let block = self.read_block(true)?;
        Ok(Node::Statement(Statement::For(
            ident.into(),
            Box::new(expr),
            Box::new(block)
        )))
    }

    fn read_let(&mut self) -> Result<Node> {
        let ident = self.read_ident_as_string()?;
        self.tokens.expect(&Token::Operator(Operator::SetValue))?;
        let expr = self.read_expression()?;
        self.expect_semi()?;
        Ok(Node::Statement(Statement::DefineVariable(
            ident.to_string(),
            Box::new(expr)
        )))
    }

    fn read_func(&mut self) -> Result<Node> {
        let ident = self.read_ident_as_string()?;
        self.tokens.expect(&Token::Operator(Operator::ParenOpen))?;
        let mut args = Vec::new();
        for arg in self.read_args(Operator::ParenClose)? {
            if let Node::Identifier(ident) = arg {
                args.push(ident);
            } else {
                return Err("expected identifier".into())
            }
        }
        let Node::Block(block) = self.read_block(true)?
            else { unreachable!() };
        Ok(Node::Statement(Statement::DefineFunction(
            ident.to_string(),
            args,
            block
        )))
    }

    fn expect_semi(&mut self) -> Result<()> {
        self.tokens.expect(&Token::Operator(Operator::Semicolon))
    }

    fn read_block(&mut self, inner: bool) -> Result<Node> {
        let mut nodes = Vec::new();

        if inner {
            self.tokens.expect(&Token::Operator(Operator::BraceOpen))?;
        }

        loop {
            match self.tokens.peek().cloned() {
                Ok(Token::Operator(Operator::BraceClose)) => {
                    if inner {
                        self.tokens.step();
                        break
                    } else {
                        return Err("unexpected '}'".into())
                    }
                }
                Ok(Token::Keyword(kw)) => {
                    self.tokens.step();
                    nodes.push(match kw {
                        If => self.read_if()?,
                        For => self.read_for()?,
                        Func => self.read_func()?,
                        Let => self.read_let()?,
                        _ => {
                            return Err("not yet implemented".into())
                        }
                    });
                }
                Ok(_) => {
                    nodes.push(self.read_expression()?);
                    self.expect_semi()?;
                }
                Err(_) => {
                    break
                }
            }
        }

        Ok(Node::Block(Block {
            nodes,
        }))
    }

    pub fn parse(&mut self) -> Result<Node> {
        self.read_block(false)
    }
}
