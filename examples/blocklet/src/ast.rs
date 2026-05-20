use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Expr {
    Num(i64),
    Var(String),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Let {
        bindings: Vec<(String, Expr)>,
        body: Box<Expr>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum EvalError {
    #[error("unbound variable: {0}")]
    Unbound(String),
    #[error("division by zero")]
    DivZero,
}

impl Expr {
    pub fn eval(&self) -> Result<i64, EvalError> {
        self.eval_env(&HashMap::new())
    }

    fn eval_env(&self, env: &HashMap<String, i64>) -> Result<i64, EvalError> {
        match self {
            Self::Num(n) => Ok(*n),
            Self::Var(name) => env
                .get(name)
                .copied()
                .ok_or_else(|| EvalError::Unbound(name.clone())),
            Self::Bin(op, l, r) => {
                let lv = l.eval_env(env)?;
                let rv = r.eval_env(env)?;
                match op {
                    BinOp::Add => Ok(lv + rv),
                    BinOp::Sub => Ok(lv - rv),
                    BinOp::Mul => Ok(lv * rv),
                    BinOp::Div => {
                        if rv == 0 {
                            Err(EvalError::DivZero)
                        } else {
                            Ok(lv / rv)
                        }
                    }
                }
            }
            Self::Let { bindings, body } => {
                let mut inner = env.clone();
                for (name, expr) in bindings {
                    let v = expr.eval_env(&inner)?;
                    inner.insert(name.clone(), v);
                }
                body.eval_env(&inner)
            }
        }
    }
}
