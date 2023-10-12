use crate::{
    builtin::builtins::{self, *},
    parser::ast::*,
    util::*,
};

use super::{
    enviroment::Environment,
    object::{self, *},
};

pub struct Evaluator {
    program: Program,
    env: Environment,
}

impl Evaluator {
    pub fn new(program: Program) -> Self {
        let env = Environment::new();
        Self { program, env }
    }

    fn eval(&mut self, statement: &Statement) -> Object {
        self.eval_statement(statement, false)
    }

    pub fn eval_program(&mut self) -> Option<Object> {
        let mut result = Some(Object::None(NoneLit));
        for statement in &self.program.statements.clone() {
            result = Some(self.eval(statement));
        }
        result
    }

    fn eval_statement(&mut self, statement: &Statement, is_local: bool) -> Object {
        match statement {
            Statement::VAR(var) => {
                let val = self.eval_expression(&var.value);
                self.env.set(&var.name.value, &val, is_local, false);
                Object::Var(Var {
                    value: Box::from(val),
                    is_local,
                })
            }
            Statement::CONST(constant) => {
                let val = self.eval_expression(&constant.value);
                self.env.set(&constant.name.value, &val, is_local, true);
                Object::Var(Var {
                    value: Box::from(val),
                    is_local,
                })
            }
            Statement::RETURN(ret) => {
                return Object::Return(Return {
                    value: Box::new(self.eval_expression(&ret.return_value)),
                });
            }
            Statement::LOCAL(stmt) => match &*stmt.left {
                Statement::VAR(_) => self.eval_statement(&*stmt.left, true),
                Statement::CONST(_) => self.eval_statement(&*stmt.left, true),
                Statement::RETURN(_) => todo!(),
                Statement::LOCAL(_) => todo!(),
                Statement::EXPRESSION(_) => todo!(),
                Statement::EMPTY => todo!(),
                Statement::USE(_) => todo!(),
            },
            Statement::EXPRESSION(expr) => self.eval_expression(&expr.expression),
            Statement::EMPTY => todo!(),
            Statement::USE(use_stmt) => self.eval_use_stmt(use_stmt),
        }
    }

    fn eval_expression(&mut self, expression: &Expression) -> Object {
        match expression {
            Expression::IDENTIFIER(ident) => self.eval_identifier(ident),
            Expression::NUMBERLITERAL(num) => Object::Num(Num { value: num.value }),
            Expression::STRINGLITERAL(str) => self.eval_string_literal(str),
            Expression::PREFIX(prefix) => self.eval_prefix_expression(prefix),
            Expression::INFIX(infix) => self.eval_infix_expression(infix),
            Expression::BOOLEAN(bool) => Object::Bool(Bool {
                value: bool.bool_type.clone(),
            }),
            Expression::IF(if_expr) => self.eval_if_expression(if_expr),
            Expression::WHILE(while_loop) => self.eval_while_expression(while_loop),
            Expression::FOR(for_loop) => self.eval_for_expression(for_loop),
            Expression::FUNC(func) => self.eval_func_expression(func),
            Expression::CALL(call) => self.eval_call(call),
            Expression::LIST(list) => self.eval_list_literal(list),
            Expression::INDEX(index) => self.eval_index(index),
            Expression::ANNOTATION(_) => todo!(),
            Expression::NONE(_) => Object::None(NoneLit),
            Expression::EMPTY => {
                Object::Error(Error::new("Cannot evaluate EMPTY expression".to_string()))
            }
            Expression::WHEN(when) => self.eval_when_expression(when),
        }
    }

    fn eval_identifier(&mut self, ident: &Identifier) -> Object {
        match self.env.get(&ident.value) {
            Ok(obj) => obj.obj,
            Err(_) => match &ident.value {
                i if i == &BuiltinType::BOOLEAN.literal() => {
                    Object::Type(object::Type::BUILTIN(BuiltinType::BOOLEAN))
                }
                i if i == &BuiltinType::NUMBER.literal() => {
                    Object::Type(object::Type::BUILTIN(BuiltinType::NUMBER))
                }
                i if i == &BuiltinType::STRING.literal() => {
                    Object::Type(object::Type::BUILTIN(BuiltinType::STRING))
                }
                _ => {
                    let err = Error::new(format!("Cannot find identifier: {}", ident.value));
                    throw_error(&err);
                    Object::Error(err)
                }
            },
        }
    }

    fn eval_use_stmt(&mut self, node: &UseStatement) -> Object {
        Object::Use(Use { file_path: convert_path(&node.path), alias: None })
    }

    // TODO: Correct formatting. Example: "{x} is cool" -> " is cool{x}"
    fn eval_string_literal(&mut self, node: &StringLiteral) -> Object {
        let mut char_stream: Vec<char> = node.value.chars().collect();
        let mut ref_pos = 0;
        let mut c_stream_pos = 0;
        let mut literal_references: Vec<String> = Vec::new();

        {
            let references = &node.references;
            references
                .iter()
                .for_each(|x| literal_references.push(self.eval_expression(x).literal()));
        }

        while c_stream_pos < char_stream.len() {
            let mut cur_ref: Vec<char> = Vec::new();
            if node.references.len() > 0 {
                cur_ref = literal_references[ref_pos].chars().collect();
            }

            cur_ref.reverse();

            if char_stream[c_stream_pos] == '{' {
                char_stream.remove(c_stream_pos);
                char_stream.remove(c_stream_pos);
                cur_ref
                    .iter()
                    .for_each(|x| char_stream.insert(c_stream_pos, *x));
                if ref_pos + 1 < node.references.len() {
                    ref_pos += 1;
                }
            }

            c_stream_pos += 1;
        }

        let str: String = char_stream.into_iter().collect();

        Object::Str(Str { value: str })
    }

    fn eval_prefix_expression(&mut self, node: &PrefixExpression) -> Object {
        let right = self.eval_expression(&node.right);
        // TODO: error checking

        match node.operator {
            Operator::BANG => self.eval_bang_expression(right),
            Operator::PLUS => right,
            Operator::MINUS => self.eval_minus_expression(right),
            _ => Object::Error(Error::new(format!(
                "Illegal prefix operation: {:?}",
                node.operator
            ))),
        }
    }

    fn eval_infix_expression(&mut self, node: &InfixExpression) -> Object {
        let left = self.eval_expression(&node.left);
        let right = self.eval_expression(&node.right);
        let operator = &node.operator;

        if left.get_type() == ObjectType::NUMBER
            && right.get_type() == ObjectType::NUMBER
            && operator != &Operator::ASSIGN
        {
            return self.eval_integer_infix_expression(operator, left, right);
        } else {
            return match operator {
                Operator::EQUAL => self.native_bool_to_object(left == right),
                Operator::NOTEQUAL => self.native_bool_to_object(left != right),
                Operator::AS => self.eval_conversion_infix_expression(node, left),
                Operator::RANGE => Object::Range(Range {
                    left: Box::from(left),
                    right: Box::from(right),
                }),
                Operator::ASSIGN => self.eval_assign_infix_expression(node, right),
                _ => Object::Error(Error::new(format!(
                    "Unknown operation: left: {:?}, right: {:?}, operator: {:?}",
                    left, right, operator
                ))),
            };
        }
    }

    fn eval_assign_infix_expression(&mut self, node: &InfixExpression, right: Object) -> Object {
        match &*node.left {
            Expression::IDENTIFIER(ident) => self.env.modify(&ident.value, right),
            _ => todo!(),
        }
        match self.env.get(
            &match &*node.left {
                Expression::IDENTIFIER(ident) => ident,
                _ => todo!(),
            }
            .value,
        ) {
            Ok(obj) => return obj.obj,
            Err(_) => todo!(),
        };
    }

    fn eval_conversion_infix_expression(&mut self, node: &InfixExpression, left: Object) -> Object {
        match &*node.right {
            Expression::IDENTIFIER(right) => {
                match &right.value {
                    r if r == &BuiltinType::STRING.literal() => Object::Str(Str {
                        value: left.literal(),
                    }),
                    r if r == &BuiltinType::NUMBER.literal() => Object::Num(Num {
                        value: match &left.literal().parse() {
                            Ok(num) => *num,
                            Err(_) => {
                                throw_error(&Error::new(format!("Failed to convert {} to a number. This value cannot be convertes", &left.literal())));
                                0f64
                            }
                        },
                    }),
                    r if r == &BuiltinType::BOOLEAN.literal() => Object::Bool(Bool {
                        value: match left.literal().as_str() {
                            "true" => BooleanType::TRUE,
                            "false" => BooleanType::FALSE,
                            _ => {
                                throw_error(&Error::new(format!(
                                    "Failed to convert {} to a boolean (true or false)",
                                    left.literal()
                                )));
                                BooleanType::FALSE
                            }
                        },
                    }),
                    _ => todo!("implement support for self defined types"),
                }
            }
            _ => todo!(),
        }
    }

    fn eval_integer_infix_expression(
        &mut self,
        operator: &Operator,
        left: Object,
        right: Object,
    ) -> Object {
        let left_val: f64;
        let right_val: f64;
        if let Object::Num(num) = left {
            left_val = num.value;
        } else {
            return Object::Error(Error::new(format!(
                "left value is not a number. Expected number found: {:?} instead",
                left
            )));
        }

        if let Object::Num(num) = right {
            right_val = num.value;
        } else {
            return Object::Error(Error::new(format!(
                "right value is not a number. Expected number found: {:?} instead",
                right
            )));
        }

        match operator {
            Operator::PLUS => Object::Num(Num {
                value: left_val + right_val,
            }),
            Operator::MINUS => Object::Num(Num {
                value: left_val - right_val,
            }),
            Operator::MULTIPLY => Object::Num(Num {
                value: left_val * right_val,
            }),
            Operator::DIVIDE => Object::Num(Num {
                value: left_val / right_val,
            }),
            Operator::GREATTHAN => self.native_bool_to_object(left_val > right_val),
            Operator::LESSTHAN => self.native_bool_to_object(left_val < right_val),
            Operator::GREATOREQUAL => self.native_bool_to_object(left_val >= right_val),
            Operator::LESSOREQUAL => self.native_bool_to_object(left_val <= right_val),
            Operator::EQUAL => self.native_bool_to_object(left_val == right_val),
            Operator::NOTEQUAL => self.native_bool_to_object(left_val != right_val),
            Operator::RANGE => Object::Range(Range {
                left: Box::from(Object::Num(Num::new(left_val))),
                right: Box::from(Object::Num(Num::new(right_val))),
            }),
            _ => Object::None(NoneLit),
        }
    }

    fn eval_block_statement(&mut self, block: &BlockStatement) -> Object {
        let mut result = Object::None(NoneLit);

        for stmt in block.statements.iter() {
            result = self.eval_statement(stmt, false);

            match result {
                Object::Return(ret) => return *ret.value,
                _ => continue,
            }
        }

        result
    }

    fn eval_if_expression(&mut self, node: &IfExpression) -> Object {
        // sussy unweap
        let condition = match &node.condition.clone() {
            Some(condition) => self.eval_expression(&condition),
            None => Object::None(NoneLit),
        }; // &node.condition.as_ref().clone().unwrap()

        if condition != Object::None(NoneLit) && self.is_truthy(&condition) {
            return self.eval_block_statement(&node.consequence);
        } else if node.alternative != None {
            return self.eval_else_expression(&node.alternative.as_ref().unwrap());
        } else {
            Object::UnMetExpr(UnmetExpr)
        }
    }

    fn eval_else_expression(&mut self, alternative: &Box<IfExpression>) -> Object {
        let alt = *alternative.clone();
        let condition = match &alt.condition.clone() {
            Some(cond) => self.eval_expression(cond),
            None => Object::None(NoneLit),
        };

        if alt.if_type == IfType::ELSE
            || alt.if_type == IfType::ELSEIF && self.is_truthy(&condition)
        {
            return self.eval_block_statement(&alternative.consequence);
        } else if alternative.alternative != None {
            return self.eval_else_expression(&alternative.alternative.as_ref().unwrap());
        } else {
            Object::UnMetExpr(UnmetExpr)
        }
    }

    fn eval_list_literal(&mut self, node: &ListExpression) -> Object {
        let mut content: Vec<Object> = Vec::new();
        node.content
            .iter()
            .for_each(|entry| content.push(self.eval_expression(entry)));

        Object::List(List {
            content,
            length: node.length,
        })
    }

    fn eval_when_expression(&mut self, node: &WhenExpression) -> Object {
        let compare_value = &*node.value;

        let cases = &*node.cases;

        let mut block: Object = Object::None(NoneLit);

        for case in cases {
            if self.eval_expression(compare_value) == self.eval_expression(&case.case_condition) {
                block = self.eval_block_statement(&case.case_consequence);
            }
        }
        block
    }

    fn eval_while_expression(&mut self, node: &WhileExpression) -> Object {
        let condition = self.eval_expression(&*node.condition);
        while self.is_truthy(&condition) {
            self.eval_block_statement(&node.consequence);
        }
        self.eval_block_statement(&node.consequence)
    }

    fn eval_for_expression(&mut self, node: &ForExpression) -> Object {
        let range = self.eval_expression(&node.loop_list);
        let range_lit = match range {
            Object::Range(range) => range,
            _ => todo!("{:?}", range),
        };

        let range_left = match *range_lit.left {
            Object::Num(num) => num,
            _ => todo!(),
        };

        let range_right = match *range_lit.right {
            Object::Num(num) => num,
            _ => todo!(),
        };

        for _ in range_left.value as i32..range_right.value as i32 - 1 {
            self.eval_block_statement(&node.consequence);
        }
        return self.eval_block_statement(&node.consequence);
    }

    fn eval_index(&mut self, node: &IndexExpression) -> Object {
        let list = self.eval_expression(&*node.list);
        match list {
            Object::List(list_obj) => list_obj
                .content
                .get(match self.eval_expression(&node.index) {
                    Object::Num(num) => num.value as usize,
                    _ => todo!(),
                })
                .unwrap()
                .clone(),
            _ => todo!(),
        }
    }

    fn eval_call(&mut self, node: &CallExpression) -> Object {
        match *node.function.clone() {
            Expression::IDENTIFIER(ident) => match ident.value {
                i if i == builtins::BuiltinFunction::PRINT.literal() => {
                    let mut args: Vec<Object> = Vec::new();
                    for arg in &node.args {
                        let evaluated_arg = self.eval_expression(&arg);
                        args.push(evaluated_arg)
                    }
                    let func = BuiltInFunction {
                        func: builtins::BuiltinFunction::PRINT,
                        args,
                    };
                    builtins::BuiltinFunction::print_val(&func);
                    Object::BuiltInFunction(func)
                }
                i if i == builtins::BuiltinFunction::INPUT.literal() => {
                    let mut args: Vec<Object> = Vec::new();
                    for arg in &node.args {
                        let evaluated_arg = self.eval_expression(&arg);
                        args.push(evaluated_arg)
                    }
                    let func = BuiltInFunction {
                        func: builtins::BuiltinFunction::PRINT,
                        args,
                    };
                    builtins::BuiltinFunction::read_input(&func);
                    Object::BuiltInFunction(func)
                }
                _ => {
                    let mut old_env = self.env.clone();
                    let new_env = self.env.clone();

                    self.env = new_env.clone();

                    let func = match self.env.get(&ident.value) {
                        Ok(obj) => obj,
                        Err(_) => {
                            let err = Error::new(format!(
                                "Cannot find identifier: {}",
                                ident.value.as_str()
                            ));
                            throw_error(&err);
                            return Object::Error(err);
                        }
                    };

                    let func_obj = if let Object::Function(func) = func.obj.clone() {
                        func
                    } else {
                        throw_error(&Error {
                            message: "Identifier is not a function".to_string(),
                        });
                        Function::empty()
                    };

                    for (index, arg) in func_obj.args.iter().enumerate() {
                        if index < node.args.len() {
                            let cur_arg = self.eval_expression(&node.args[index]);
                            self.env.set(&arg.arg.value, &cur_arg, false, false);
                        } else {
                            break;
                        }
                    }

                    for stmt in func_obj.body.statements {
                        let obj = self.eval(&stmt);
                        match obj {
                            Object::Return(ret) => {
                                return *ret.value;
                            }
                            _ => continue,
                        }
                    }

                    // Update all args cuz of mutability
                    for (index, arg) in node.args.iter().enumerate() {
                        let call_arg_name = match arg {
                            Expression::IDENTIFIER(ident) => Some(&ident.value),
                            _ => None,
                        };

                        let func_arg_name = &func_obj.args[index].arg.value;

                        match call_arg_name {
                            Some(arg_name) => {
                                old_env.modify(
                                    arg_name,
                                    match self.env.get(func_arg_name) {
                                        Ok(env_obj) => env_obj.obj,
                                        Err(_) => todo!(),
                                    },
                                );
                            }
                            None => (),
                        }

                        // Take call args and update related vars with values from function
                    }

                    self.env = old_env;

                    func.obj
                }
            },
            _ => todo!("{:?}", node.function),
        }
    }

    fn eval_func_expression(&mut self, node: &FuncExpression) -> Object {
        Object::Function(Function {
            args: node.args.to_owned(),
            body: node.body.to_owned(),
        })
    }

    fn is_truthy(&mut self, object: &Object) -> bool {
        match object {
            Object::Bool(bool) => match bool.value {
                BooleanType::TRUE => true,
                BooleanType::FALSE => false,
            },
            Object::None(_) => false,
            _ => {
                throw_error(&Error::new(format!(
                    "Invalid condition: {}",
                    object.literal()
                )));
                // this will not be returned as throw_error()
                // will terminate the process
                false
            }
        }
    }

    fn native_bool_to_object(&self, bool: bool) -> Object {
        match bool {
            true => Object::Bool(Bool {
                value: BooleanType::TRUE,
            }),
            false => Object::Bool(Bool {
                value: BooleanType::FALSE,
            }),
        }
    }

    fn eval_bang_expression(&self, right: Object) -> Object {
        match right {
            Object::Bool(obj) => match obj.value {
                BooleanType::TRUE => Object::Bool(Bool {
                    value: BooleanType::FALSE,
                }),
                BooleanType::FALSE => Object::Bool(Bool {
                    value: BooleanType::TRUE,
                }),
            },
            Object::None(_) => right,
            _ => todo!(),
        }
    }

    fn eval_minus_expression(&self, right: Object) -> Object {
        match right {
            Object::Num(num) => Object::Num(Num { value: -num.value }),
            _ => right,
        }
    }
}