pub mod dice;

use rand::rngs::ThreadRng;

use std::error;

#[derive(Debug)]
pub enum Operator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

impl Operator {
    fn from(string: &String) -> Result<Operator, &'static str> {
        match string.as_str() {
            "+" => Ok(Operator::Addition),
            "-" => Ok(Operator::Subtraction),
            "*" => Ok(Operator::Multiplication),
            "/" => Ok(Operator::Division),
            _ => Err("Unkown operator in expression (Something is seriously wrong here.)"),
        }
    }
    
    // TODO: Deal with NaNs and overflows
    fn operate(&self, lhs: u32, rhs: u32) -> u32 {
        match self {
            Operator::Addition => lhs + rhs,
            Operator::Subtraction => lhs.checked_sub(rhs).unwrap_or(0),
            Operator::Multiplication => lhs * rhs,
            Operator::Division => lhs.checked_div(rhs).unwrap_or(0),
        }
    }

    fn precedence(op: &str) -> u32{
        match op {
            "+" => 0,
            "-" => 0,
            "*" => 1,
            "/" => 1,
            _ => 0,
        }
    }
}

#[derive(Debug)]
pub enum Operand {
    Number(u32),
    Dice(dice::Dice),
    Roll(dice::Roll),
}

impl Operand {
    fn from(string: &str) -> Result<Operand, &str> {
        if string.contains('d') {
            if let Ok(result) = dice::Dice::new(string) {
                Ok(Operand::Dice(result))
            } else {
                Err("Could not parse dice")
            }
        } else {
            if let Ok(num) = string.parse::<u32>() {
                Ok(Operand::Number(num))
            } else {
                Err("Could not parse as dice or number")
            }
        }
    }
}

#[derive(Debug)]
pub enum Element {
    OperatorEl(Operator),
    OperandEl(Operand)
}

pub struct Expression {
    right: Option<Box<Expression>>,
    left: Option<Box<Expression>>,
    element: Element,
}

impl Expression {
    pub fn from_str(infix_str: &str) -> Result<Expression, &str> {
        let builder = ExpressionBuilder::new();
        builder.build(infix_str)
    }

    fn from_stack(expr_stack: &mut Vec<Element>) -> Option<Box<Expression>> {
        if let Some(element) = expr_stack.pop() {
            match element {
                Element::OperatorEl(_) => Some(Box::new(Expression {
                    element: element,
                    right: Self::from_stack(expr_stack),
                    left: Self::from_stack(expr_stack),
                })),
                Element::OperandEl(_) => Some(Box::new(Expression {
                    element: element,
                    right: None,
                    left: None,
                }))
            }
        } else {
            None
        }
    }
    
    pub fn determine(&mut self, rng: &mut ThreadRng) {
        if let Element::OperandEl(Operand::Dice(dice)) = &mut self.element {
            self.element = Element::OperandEl(Operand::Roll(dice.roll(rng)));
        }
        
        if let Some(left) = &mut self.left {
            (*left).determine(rng);
        }

        if let Some(right) = &mut self.right {
            (*right).determine(rng);
        }
    }

    pub fn resolve(&self, rng: &mut ThreadRng) -> Result<u32, Box<dyn error::Error>> {
        match &self.element {
            Element::OperandEl(operand) => {
                match operand {
                    Operand::Number(num) => Ok(*num),
                    Operand::Dice(dice) => Ok(dice.roll(rng).result),
                    Operand::Roll(roll) => Ok(roll.result),
                }
            },
            Element::OperatorEl(operator) => {
                if let (Some(left), Some(right)) = (self.left.as_ref(), self.right.as_ref()) {
                    let lhs = (*left).resolve(rng)?;
                    let rhs = (*right).resolve(rng)?;
                    Ok(operator.operate(lhs, rhs))
                } else {
                    Err("Malformed tree, operand(s) absent for binary operator".into())
                }
            },
        }
    }
}

struct ExpressionBuilder {
    op_stack: Vec<String>,
    expr_stack: Vec<Element>,
}

impl ExpressionBuilder {
    fn build(mut self, expr_str: &str) -> Result<Expression, &str> {
        let mut last_index: usize = 0;
        let iter =  expr_str.match_indices(|c: char| -> bool {
            ['+', '-', '*', '/', '(', ')'].contains(&c)
        });
        
        for (index, operator_str) in iter {
            // Add the last operand to the expression stack
            let last_operand_str = expr_str.get(last_index..index).unwrap_or("");
            self.try_push_operand(last_operand_str);

            // Handle operators including "(" which has special handling in try_push_operator
            match operator_str {
                ")" => {self.close_paren()?;},
                _ => {self.try_push_operator(operator_str)?;},
            };

            last_index = index + 1;

            #[cfg(debug_assertions)]
            println!("Stack: {:?}", self.op_stack);
            #[cfg(debug_assertions)]
            println!("Expre: {:?}", self.expr_stack);
        }

        // Catch trailing operand
        let final_operand_str = expr_str.get(last_index..).unwrap_or("");
        self.try_push_operand(final_operand_str);

        // Clear operator stack to expression stack
        self.shuffle_ops_until(|_op| false);

        if let Some(expr) = Expression::from_stack(&mut self.expr_stack) {
            Ok(*expr)
        } else {
            Err("Unable to create expression tree")
        }
    }
    
    fn close_paren(&mut self) -> Result<(), &'static str> {
        let mut matching = false;

        self.shuffle_ops_until(|op| -> bool {
            matching = op.as_str() == "(";
            matching
        });


        if !matching {
            Err("Unbalanced parentheses")
        } else {
            self.op_stack.pop();
            Ok(())
        }
    }

    fn new() -> ExpressionBuilder {
        ExpressionBuilder {
            op_stack: Vec::new(),
            expr_stack: Vec::new(),
        }
    }

    fn shuffle_ops_until<F>(&mut self, mut f: F) where F: FnMut(&String) -> bool {
        while let Some(op) = self.op_stack.pop() {
            if f(&op) {
                self.op_stack.push(op);
                break;
            } else {
                self.expr_stack.push(Element::OperatorEl(Operator::from(&op).unwrap_or(Operator::Multiplication)));
            }
        }
    }

    fn try_push_operand(&mut self, operand_str: &str) {
        if operand_str != "" {
            if let Ok(op) = Operand::from(operand_str) {
                self.expr_stack.push(Element::OperandEl(op));
            }
        }
    }

    fn try_push_operator(&mut self, operator_str: &str) -> Result<(), &'static str> {
        self.shuffle_ops_until(|stacked_op| -> bool {
            if operator_str == "("
                || stacked_op == "(" {
                true
            } else {
                let lhs = Operator::precedence(operator_str);
                let rhs = Operator::precedence(stacked_op);
                lhs > rhs
            }
        });
        
        self.op_stack.push(String::from(operator_str));
        Ok(())
    }
}
